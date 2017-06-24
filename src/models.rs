use std::cell::RefCell;
use std::rc::Rc;

use comm::address::Address;
use comm;

pub trait Observable<O> {
    fn register_observer(&mut self, observer: O);
}

pub trait ConversationListObserver {
    fn conversation_was_added(&self, Rc<RefCell<Conversation>>);
    fn conversation_was_selected(&self, Rc<RefCell<Conversation>>);
}

pub trait ConversationObserver {
    fn recipient_was_changed(&self, Address);
}

pub struct Conversation {
    recipient: Option<Address>,
    pending_message: String,
    client_commands: comm::client::TaskSender,
    observers: Vec<Rc<RefCell<ConversationObserver>>>
}

impl Conversation {
    pub fn new(client_commands: comm::client::TaskSender) -> Conversation {
        Conversation {
            recipient: None,
            pending_message: String::new(),
            client_commands: client_commands,
            observers: vec![]
        }
    }

    pub fn recipient(&self) -> Option<Address> {
        self.recipient
    }

    pub fn pending_message(&self) -> &str {
        &self.pending_message
    }

    pub fn set_pending_message(&mut self, text: String) {
        self.pending_message = text;
    }

    pub fn set_recipient(&mut self, recipient: Address) {
        self.recipient = Some(recipient);
        for observer in self.observers.iter() {
            observer.borrow().recipient_was_changed(recipient);
        }
    }

    pub fn send_message(&mut self) {
        if let Some(recipient) = self.recipient {
            let text_message = comm::client::messages::TextMessage::new(recipient, self.pending_message.clone());
            self.client_commands
                .send(comm::client::Task::ScheduleMessageDelivery(recipient, text_message))
                .expect("Couldn't send message");

            self.pending_message = String::new();
        }
    }
}

impl Observable<Rc<RefCell<ConversationObserver>>> for Conversation {
    fn register_observer(&mut self, observer: Rc<RefCell<ConversationObserver>>) {
        self.observers.push(observer);
    }
}

pub struct ConversationList {
    conversations: Vec<Rc<RefCell<Conversation>>>,
    observers: Vec<Rc<RefCell<ConversationListObserver>>>
}

impl ConversationList {
    pub fn new() -> ConversationList {
        ConversationList {
            conversations: vec![],
            observers: vec![]
        }
    }

    pub fn prepend(&mut self, conversation: Rc<RefCell<Conversation>>) {
        self.conversations.insert(0, conversation.clone());
        for observer in self.observers.iter() {
            observer.borrow().conversation_was_added(conversation.clone());
        }
    }

    pub fn get(&self, index: usize) -> Option<&Rc<RefCell<Conversation>>> {
        self.conversations.get(index)
    }

    pub fn select_conversation(&self, index: usize) {
        let conversation = self.get(index).unwrap();
        for observer in self.observers.iter() {
            observer.borrow().conversation_was_selected(conversation.clone());
        }
    }
}

impl Observable<Rc<RefCell<ConversationListObserver>>> for ConversationList {
    fn register_observer(&mut self, observer: Rc<RefCell<ConversationListObserver>>) {
        self.observers.push(observer);
    }
}
