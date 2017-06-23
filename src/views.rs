use gtk;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use comm::address;

use models;

pub struct Conversation {
    address: gtk::Entry,
    widget: gtk::Box
}

impl Conversation {
    pub fn new() -> Conversation {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let address = gtk::Entry::new();
        let transcript = gtk::Stack::new();
        let message = gtk::Entry::new();
        widget.pack_start(&address, false, false, 0);
        widget.pack_start(&transcript, true, true, 0);
        widget.pack_start(&message, false, false, 0);

        Conversation {
            address: address,
            widget: widget
        }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }

    pub fn set_conversation(&self, conversation: &Rc<RefCell<models::Conversation>>) {
        // Initialize UI widgets
        match conversation.borrow().recipient() {
            Some(address) => self.address.set_text(&address.to_str()),
            None => self.address.delete_text(0, -1)
        }

        // Connection UI events
        let c = conversation.clone();
        self.address.connect_preedit_changed(move |entry, _| {
            let text = entry.get_text().unwrap();
            if text.len() == 40 {
                let address = address::Address::from_str(&text);
                c.borrow_mut().set_recipient(address);
            }
        });
    }
}

pub struct ConversationListItem {
    widget: gtk::ListBoxRow
}

impl ConversationListItem {
    pub fn new(_conversation: Rc<RefCell<models::Conversation>>) -> ConversationListItem {
        // TODO: connect some event listener thing on the conversation
        // to update this view when it changes.
        let widget = gtk::ListBoxRow::new();
        let label = gtk::Label::new_with_mnemonic(Some("Conversation"));
        widget.add(&label);

        ConversationListItem {
            widget: widget
        }
    }

    pub fn widget(&self) -> &gtk::ListBoxRow {
        &self.widget
    }
}

pub struct ConversationList {
    repo: Rc<RefCell<models::ConversationList>>,
    widget: gtk::ListBox
}

impl ConversationList {
    pub fn new(repo: Rc<RefCell<models::ConversationList>>) -> ConversationList {
        ConversationList {
            repo: repo,
            widget: gtk::ListBox::new()
        }
    }

    pub fn add_conversation(&self, conversation: Rc<RefCell<models::Conversation>>) {
        self.repo.borrow_mut().prepend(conversation.clone());
        let list_item = ConversationListItem::new(conversation.clone());
        self.widget.prepend(list_item.widget());
        list_item.widget().show_all();
    }

    pub fn widget(&self) -> &gtk::ListBox {
        &self.widget
    }
}
