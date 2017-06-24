use gtk;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use comm::address;

use models;
use models::{Observable, ConversationListObserver};

pub struct Conversation {
    address: gtk::Entry,
    view: gtk::Box,
    message: gtk::Entry,
    send_button: gtk::Button
}

impl Conversation {
    pub fn new() -> Conversation {
        let view = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let address = gtk::Entry::new();
        let transcript = gtk::Stack::new();
        let send_button = gtk::Button::new_with_label("Send");
        let message = gtk::Entry::new();
        let send_pane = gtk::Paned::new(gtk::Orientation::Horizontal);
        send_pane.set_position(300);
        send_pane.add1(&message);
        send_pane.add2(&send_button);
        view.pack_start(&address, false, false, 0);
        view.pack_start(&transcript, true, true, 0);
        view.pack_start(&send_pane, false, false, 0);

        Conversation {
            address: address,
            view: view,
            message: message,
            send_button: send_button
        }
    }

    pub fn view(&self) -> &gtk::Box {
        &self.view
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
    view: gtk::ListBoxRow
}

impl ConversationListItem {
    pub fn new(_conversation: Rc<RefCell<models::Conversation>>) -> ConversationListItem {
        // TODO: connect some event listener thing on the conversation
        // to update this view when it changes.
        let view = gtk::ListBoxRow::new();
        let label = gtk::Label::new_with_mnemonic(Some("Conversation"));
        view.add(&label);

        ConversationListItem {
            view: view
        }
    }

    pub fn view(&self) -> &gtk::ListBoxRow {
        &self.view
    }
}

pub struct ConversationList {
    model: Rc<RefCell<models::ConversationList>>,
    view: gtk::ListBox
}

impl ConversationList {
    pub fn new(model: Rc<RefCell<models::ConversationList>>) -> Rc<RefCell<ConversationList>> {
        let controller = Rc::new(RefCell::new(ConversationList {
            model: model.clone(),
            view: gtk::ListBox::new()
        }));

        let c = controller.clone();
        controller.borrow().view().connect_row_selected(move |_, list_item| {
            let index = list_item.as_ref().unwrap().get_index() as usize;
            c.borrow().select_conversation(index);
        });

        model.borrow_mut().register_observer(controller.clone());

        controller
    }

    pub fn add_conversation(&self, conversation: Rc<RefCell<models::Conversation>>) {
        self.model.borrow_mut().prepend(conversation.clone());
    }

    pub fn select_conversation(&self, index: usize) {
        self.model.borrow().select_conversation(index);
    }

    pub fn view(&self) -> &gtk::ListBox {
        &self.view
    }
}

impl ConversationListObserver for ConversationList {
    fn conversation_was_added(&self, conversation: Rc<RefCell<models::Conversation>>) {
        let list_item = ConversationListItem::new(conversation);
        self.view.prepend(list_item.view());
        list_item.view().show_all();
    }

    fn conversation_was_selected(&self, conversation: Rc<RefCell<models::Conversation>>) {
        println!("conversation_was_selected: {:?}", conversation);
    }
}
