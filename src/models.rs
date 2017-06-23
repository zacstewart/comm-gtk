use std::cell::RefCell;
use std::rc::Rc;

use comm::address::Address;

#[derive(Debug)]
pub struct Conversation {
    recipient: Option<Address>
}

impl Conversation {
    pub fn new() -> Conversation {
        Conversation {
            recipient: None
        }
    }

    pub fn recipient(&self) -> Option<Address> {
        self.recipient
    }

    pub fn set_recipient(&mut self, recipient: Address) {
        self.recipient = Some(recipient);
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
