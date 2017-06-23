use gtk;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use comm::address;

use models;

pub struct Conversation {
    address: gtk::Entry,
    widget: gtk::Box,
    message: gtk::Entry,
    send_button: gtk::Button
}

impl Conversation {
    pub fn new() -> Conversation {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let address = gtk::Entry::new();
        let transcript = gtk::Stack::new();
        let send_button = gtk::Button::new_with_label("Send");
        let message = gtk::Entry::new();
        let send_pane = gtk::Paned::new(gtk::Orientation::Horizontal);
        send_pane.set_position(300);
        send_pane.add1(&message);
        send_pane.add2(&send_button);
        widget.pack_start(&address, false, false, 0);
        widget.pack_start(&transcript, true, true, 0);
        widget.pack_start(&send_pane, false, false, 0);

        Conversation {
            address: address,
            widget: widget,
            message: message,
            send_button: send_button
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
        self.message.set_text(conversation.borrow().pending_message());

        // Connection UI events
        let c = conversation.clone();
        self.address.connect_preedit_changed(move |entry, _| {
            let text = entry.get_text().unwrap();
            if text.len() == 40 {
                let address = address::Address::from_str(&text);
                c.borrow_mut().set_recipient(address);
            }
        });

        let c = conversation.clone();
        self.message.connect_preedit_changed(move |entry, _| {
            let text = entry.get_text().unwrap();
            c.borrow_mut().set_pending_message(text);
        });

        let c = conversation.clone();
        self.send_button.connect_clicked(move |_| {
            c.borrow_mut().send_message();
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
