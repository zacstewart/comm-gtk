use gtk::prelude::*;
use gtk;
use std::cell::RefCell;
use std::rc::Rc;

use comm;
use comm::address;

use models;
use models::{ConversationListObserver, ConversationObserver, Observable};

pub struct ConversationRecipient {
    view: gtk::Entry
}

impl ConversationRecipient {
    pub fn new(conversation: Rc<RefCell<models::Conversation>>) -> Rc<RefCell<ConversationRecipient>> {
        let view = gtk::Entry::new();

        let controller = Rc::new(RefCell::new(ConversationRecipient {
            view: view
        }));

        conversation.borrow_mut().register_observer(controller.clone());

        match conversation.borrow().recipient() {
            Some(address) => controller.borrow().view().set_text(&address.to_str()),
            None => controller.borrow().view().set_text("New Conversation")
        }

        controller.borrow().view().connect_changed(move |entry| {
            let text = entry.get_text().unwrap();
            if text.len() == 40 {
                let address = address::Address::from_str(&text);
                conversation.borrow_mut().set_recipient(address);
            }
        });

        controller
    }

    pub fn view(&self) -> &gtk::Entry {
        &self.view
    }
}

impl ConversationObserver for ConversationRecipient {
    fn recipient_was_changed(&self, address: comm::address::Address) {
        self.view.set_text(&address.to_str());
    }

    fn pending_message_was_changed(&self, _: String) {
    }

    fn did_receive_message(&self, _: Rc<RefCell<models::Message>>) {
    }

    fn did_send_message(&self, _: Rc<RefCell<models::Message>>) {
    }
}

pub struct Message {
    view: gtk::Label
}

impl Message {
    pub fn new(message: Rc<RefCell<models::Message>>) -> Rc<RefCell<Message>> {
        let view = gtk::Label::new(Some(message.borrow().text()));
        if message.borrow().was_sent() {
            view.set_halign(gtk::Align::End);
        } else {
            view.set_halign(gtk::Align::Start);
        }

        let controller = Rc::new(RefCell::new(Message {
            view: view
        }));

        controller
    }

    pub fn view(&self) -> &gtk::Label {
        &self.view
    }
}

pub struct Transcript {
    view: gtk::ScrolledWindow,
    container: gtk::Box
}

impl Transcript {
    pub fn new(conversation: Rc<RefCell<models::Conversation>>) -> Rc<RefCell<Transcript>> {
        let view = gtk::ScrolledWindow::new(None, None);
        let viewport = gtk::Viewport::new(None, None);
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        viewport.add(&container);
        view.add(&viewport);

        let controller = Rc::new(RefCell::new(Transcript {
            view: view,
            container: container
        }));

        conversation.borrow_mut().register_observer(controller.clone());

        controller
    }

    pub fn view(&self) -> &gtk::ScrolledWindow {
        &self.view
    }
}

impl ConversationObserver for Transcript {
    fn recipient_was_changed(&self, _: comm::address::Address) {
    }

    fn pending_message_was_changed(&self, _: String) {
    }

    fn did_receive_message(&self, message: Rc<RefCell<models::Message>>) {
        let message_controller = Message::new(message);
        self.container.pack_start(message_controller.borrow().view(), false, false, 0);
        self.view().show_all();
    }

    fn did_send_message(&self, message: Rc<RefCell<models::Message>>) {
        let message_controller = Message::new(message);
        self.container.pack_start(message_controller.borrow().view(), false, false, 0);
        self.view().show_all();
    }
}

pub struct Conversation {
    view: gtk::Box,
    pending_message: gtk::Entry
}

impl Conversation {
    pub fn new(conversation: Rc<RefCell<models::Conversation>>) -> Rc<RefCell<Conversation>> {
        let view = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let recipient_controller = ConversationRecipient::new(conversation.clone());
        let transcript_controller = Transcript::new(conversation.clone());
        let send_button = gtk::Button::new_with_label("Send");
        let pending_message = gtk::Entry::new();
        let send_pane = gtk::Paned::new(gtk::Orientation::Horizontal);
        send_pane.set_position(300);
        send_pane.add1(&pending_message);
        send_pane.add2(&send_button);
        view.pack_start(recipient_controller.borrow().view(), false, false, 0);
        view.pack_start(transcript_controller.borrow().view(), true, true, 0);
        view.pack_start(&send_pane, false, false, 0);

        for message in conversation.borrow().messages().iter().cloned() {
            transcript_controller.borrow_mut().did_receive_message(message);
        }

        let controller = Rc::new(RefCell::new(Conversation {
            view: view,
            pending_message: pending_message
        }));

        // Initialize UI widgets
        controller.borrow().pending_message().set_text(conversation.borrow().pending_message());

        // Connection UI events
        let c = conversation.clone();
        controller.borrow().pending_message().connect_preedit_changed(move |entry, _| {
            let text = entry.get_text().unwrap();
            c.borrow_mut().set_pending_message(text);
        });

        let c = conversation.clone();
        send_button.connect_clicked(move |_| {
            c.borrow_mut().send_message();
        });

        conversation.borrow_mut().register_observer(controller.clone());

        controller
    }

    pub fn pending_message(&self) -> &gtk::Entry {
        &self.pending_message
    }

    pub fn view(&self) -> &gtk::Box {
        &self.view
    }
}

impl ConversationObserver for Conversation {
    fn recipient_was_changed(&self, _: comm::address::Address) {
    }

    fn pending_message_was_changed(&self, pending_message: String) {
        self.pending_message.set_text(&pending_message)
    }

    fn did_receive_message(&self, _: Rc<RefCell<models::Message>>) {
    }

    fn did_send_message(&self, _: Rc<RefCell<models::Message>>) {
    }
}

pub struct ConversationListItemTitle {
    view: gtk::Label
}

impl ConversationListItemTitle {
    pub fn new(conversation: Rc<RefCell<models::Conversation>>) -> Rc<RefCell<ConversationListItemTitle>> {
        let view = gtk::Label::new("");
        let controller = Rc::new(RefCell::new(ConversationListItemTitle {
            view: view
        }));

        match conversation.borrow().recipient() {
            None => controller.borrow().view().set_text("New Conversation"),
            Some(address) => controller.borrow().view().set_text(&address.to_str())
        };

        conversation.borrow_mut().register_observer(controller.clone());

        controller
    }

    pub fn view(&self) -> &gtk::Label {
        &self.view
    }
}
impl ConversationObserver for ConversationListItemTitle {
    fn recipient_was_changed(&self, address: comm::address::Address) {
        self.view.set_text(&address.to_str());
    }

    fn pending_message_was_changed(&self, _: String) {
    }

    fn did_receive_message(&self, _: Rc<RefCell<models::Message>>) {
    }

    fn did_send_message(&self, _: Rc<RefCell<models::Message>>) {
    }
}

pub struct ConversationListItem {
    view: gtk::ListBoxRow
}

impl ConversationObserver for ConversationListItem {
    fn recipient_was_changed(&self, _: comm::address::Address) {
    }

    fn pending_message_was_changed(&self, _: String) {
    }

    fn did_receive_message(&self, _: Rc<RefCell<models::Message>>) {
    }

    fn did_send_message(&self, _: Rc<RefCell<models::Message>>) {
    }
}

impl ConversationListItem {
    pub fn new(conversation: Rc<RefCell<models::Conversation>>) -> Rc<RefCell<ConversationListItem>> {
        // TODO: connect some event listener thing on the conversation
        // to update this view when it changes.
        let view = gtk::ListBoxRow::new();

        let title_controller = ConversationListItemTitle::new(conversation.clone());
        view.add(title_controller.borrow().view());

        let controller = Rc::new(RefCell::new(ConversationListItem {
            view: view
        }));

        conversation.borrow_mut().register_observer(controller.clone());

        controller
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
        self.view.prepend(list_item.borrow().view());
        list_item.borrow().view().show_all();
    }

    fn conversation_was_selected(&self, _: Rc<RefCell<models::Conversation>>) {
    }
}

pub struct Conversations {
    view: gtk::Paned
}

impl Conversations {
    pub fn new(connection: Rc<models::Connection>, conversation: Rc<RefCell<models::ConversationList>>) -> Rc<RefCell<Conversations>> {
        let controller = Rc::new(RefCell::new(Conversations {
            view: gtk::Paned::new(gtk::Orientation::Horizontal)
        }));

        controller.borrow().view().set_position(200);

        let sidebar_pane = gtk::Paned::new(gtk::Orientation::Vertical);
        controller.borrow().view().add1(&sidebar_pane);
        let search_add_pane = gtk::Paned::new(gtk::Orientation::Horizontal);

        let search = gtk::SearchEntry::new();
        let new_contact_button = gtk::Button::new_from_icon_name("contact-new", 2);

        search_add_pane.pack1(&search, true, true);
        search_add_pane.pack2(&new_contact_button, false, false);

        let conversation_list_controller = ConversationList::new(conversation.clone());

        sidebar_pane.pack1(&search_add_pane, false, false);
        sidebar_pane.add2(conversation_list_controller.borrow().view());
        new_contact_button.connect_clicked(move |_| {
            let conversation = Rc::new(RefCell::new(models::Conversation::new(connection.clone())));
            conversation_list_controller.borrow().add_conversation(conversation);
        });

        conversation.borrow_mut().register_observer(controller.clone());

        controller
    }

    pub fn view(&self) -> &gtk::Paned {
        &self.view
    }
}

impl ConversationListObserver for Conversations {
    fn conversation_was_added(&self, _: Rc<RefCell<models::Conversation>>) {
    }

    fn conversation_was_selected(&self, conversation: Rc<RefCell<models::Conversation>>) {
        let conversation_controller = Conversation::new(conversation);
        if let Some(widget) = self.view.get_child2() {
            widget.destroy();
        }
        self.view.add2(conversation_controller.borrow().view());
        self.view.show_all();
    }
}
