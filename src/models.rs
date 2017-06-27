use std::cell::RefCell;
use std::sync::mpsc;
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
    fn pending_message_was_changed(&self, String);
    fn did_receive_message(&self, Rc<RefCell<Message>>);
    fn did_send_message(&self, Rc<RefCell<Message>>);
}

pub struct Connection {
    commands: comm::client::TaskSender,
    events: mpsc::Receiver<comm::client::Event>,
    self_address: comm::address::Address
}

impl Connection {
    pub fn start(secret: &str, host: &str, router: Option<&String>) -> Connection {
        let address = comm::address::Address::for_content(secret);

        let routers: Vec<Box<comm::node::Node>> = match router {
            Some(r) => {
                let router_node = Box::new(comm::node::UdpNode::new(comm::address::Address::null(), r.as_str()));
                vec![router_node]
            }
            None => vec![]
        };

        let network = comm::network::Network::new(address, host, routers);
        let mut client = comm::client::Client::new(address);
        let (event_sender, events) = mpsc::channel();
        client.register_event_listener(event_sender);
        let client_channel = client.run(network);

        Connection {
            commands: client_channel,
            events: events,
            self_address: address
        }
    }

    pub fn commands(&self) -> &comm::client::TaskSender {
        &self.commands
    }

    pub fn events(&self) -> &mpsc::Receiver<comm::client::Event> {
        &self.events
    }

    pub fn self_address(&self) -> Address {
        self.self_address
    }
}

#[derive(PartialEq, Eq)]
pub enum MessageDirection {
    Sent, Received
}

pub struct Message {
    id: Address,
    text: String,
    direction: MessageDirection
}

impl Message {
    pub fn sent(id: Address, text: String) -> Message {
        Message {
            id: id,
            text: text,
            direction: MessageDirection::Sent
        }
    }

    pub fn received(id: Address, text: String) -> Message {
        Message {
            id: id,
            text: text,
            direction: MessageDirection::Received
        }
    }

    pub fn text(&self) -> &str{
        &self.text
    }

    pub fn was_sent(&self) -> bool {
        self.direction == MessageDirection::Sent
    }

    pub fn was_received(&self) -> bool {
        self.direction == MessageDirection::Received
    }
}

pub struct Conversation {
    connection: Rc<RefCell<Connection>>,
    recipient: Option<Address>,
    pending_message: String,
    messages: Vec<Rc<RefCell<Message>>>,
    observers: Vec<Rc<RefCell<ConversationObserver>>>
}

impl Conversation {
    pub fn new(connection: Rc<RefCell<Connection>>) -> Conversation {
        Conversation {
            connection: connection,
            recipient: None,
            pending_message: String::new(),
            messages: vec![],
            observers: vec![]
        }
    }

    pub fn recipient(&self) -> Option<Address> {
        self.recipient
    }

    pub fn messages(&self) -> &Vec<Rc<RefCell<Message>>> {
        &self.messages
    }

    pub fn pending_message(&self) -> &str {
        &self.pending_message
    }

    pub fn set_pending_message(&mut self, text: String) {
        self.pending_message = text.clone();
        for observer in self.observers.iter() {
            observer.borrow().pending_message_was_changed(text.clone());
        }
    }

    pub fn set_recipient(&mut self, recipient: Address) {
        self.recipient = Some(recipient);
        for observer in self.observers.iter() {
            observer.borrow().recipient_was_changed(recipient);
        }
    }

    pub fn receive_message(&mut self, message: Rc<RefCell<Message>>) {
        self.messages.push(message.clone());
        for observer in self.observers.iter() {
            observer.borrow().did_receive_message(message.clone());
        }
    }

    pub fn send_message(&mut self) {
        if let Some(recipient) = self.recipient {
            let tm = comm::client::messages::TextMessage::new(
                self.connection.borrow().self_address(), self.pending_message.clone());

            self.connection.borrow().commands()
                .send(comm::client::Task::ScheduleMessageDelivery(recipient, tm.clone()))
                .expect("Couldn't send message");

            self.set_pending_message(String::new());

            let message = Rc::new(RefCell::new(Message::sent(tm.id, tm.text)));
            self.messages.push(message.clone());
            for observer in self.observers.iter() {
                observer.borrow().did_send_message(message.clone());
            }
        }
    }
}

impl Observable<Rc<RefCell<ConversationObserver>>> for Conversation {
    fn register_observer(&mut self, observer: Rc<RefCell<ConversationObserver>>) {
        self.observers.push(observer);
    }
}

pub struct ConversationList {
    connection: Rc<RefCell<Connection>>,
    conversations: Vec<Rc<RefCell<Conversation>>>,
    observers: Vec<Rc<RefCell<ConversationListObserver>>>
}

impl ConversationList {
    pub fn new(connection: Rc<RefCell<Connection>>) -> ConversationList {
        ConversationList {
            connection: connection,
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

    pub fn handle_event(&mut self, event: comm::client::Event) {
        match event {
            comm::client::Event::ReceivedTextMessage(tm) => {
                let sender = tm.sender;
                let message = Rc::new(RefCell::new(Message::received(tm.id, tm.text)));
                let existing_conversation = self.conversations.iter().any(|conversation| {
                    conversation.borrow().recipient() == Some(sender)
                });

                if existing_conversation {
                    let c = self.conversations.iter().find(|conversation| {
                        conversation.borrow().recipient() == Some(sender)
                    }).unwrap();
                    c.borrow_mut().receive_message(message);
                } else {
                    let c = Rc::new(RefCell::new(Conversation::new(self.connection.clone())));
                    c.borrow_mut().set_recipient(sender);
                    self.prepend(c.clone());
                    c.borrow_mut().receive_message(message);
                }
            }
            _ => { }
        }
    }
}

impl Observable<Rc<RefCell<ConversationListObserver>>> for ConversationList {
    fn register_observer(&mut self, observer: Rc<RefCell<ConversationListObserver>>) {
        self.observers.push(observer);
    }
}
