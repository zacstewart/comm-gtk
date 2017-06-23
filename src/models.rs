use std::cell::RefCell;
use std::rc::Rc;

use comm::address::Address;

#[derive(Debug)]
pub struct Conversation {
    recipient: Option<Address>,
    pending_message: String
}

impl Conversation {
    pub fn new() -> Conversation {
        Conversation {
            recipient: None,
            pending_message: String::new()
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
    }

    pub fn send_message(&mut self) {
        if let Some(recipient) = self.recipient {
            println!("[{}] -> {}", recipient, self.pending_message);
            self.pending_message = String::new();
        }
    }
}

pub struct ConversationList {
    conversations: Vec<Rc<RefCell<Conversation>>>
}

impl ConversationList {
    pub fn new() -> ConversationList {
        ConversationList {
            conversations: vec![]
        }
    }

    pub fn prepend(&mut self, conversation: Rc<RefCell<Conversation>>) {
        self.conversations.insert(0, conversation);
    }

    pub fn get(&self, index: usize) -> Option<&Rc<RefCell<Conversation>>> {
        self.conversations.get(index)
    }
}
