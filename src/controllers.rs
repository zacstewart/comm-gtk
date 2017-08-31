use glib;
use glib::signal;
use gtk::prelude::*;
use gtk;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use comm;
use comm::address;

use models;
use models::{ConversationListObserver, ConversationObserver, MessageObserver, Observable};

pub struct Configuration {
    view: gtk::Window
}

impl Configuration {
    pub fn new(configuration: Rc<RefCell<models::Configuration>>) -> Rc<RefCell<Configuration>> {
        let view = gtk::Window::new(gtk::WindowType::Toplevel);
        view.set_title("Configuration");
        view.set_position(gtk::WindowPosition::Center);
        let grid = gtk::Grid::new();

        let secret_label = gtk::Label::new("Your secret");
        secret_label.set_halign(gtk::Align::Start);
        let secret_entry = gtk::Entry::new();
        grid.attach(&secret_label, 0, 0, 100, 12);
        grid.attach_next_to(&secret_entry,
                            Some(&secret_label),
                            gtk::PositionType::Right,
                            100, 12);

        let bootstrap_label = gtk::Label::new("Bootstrap node");
        bootstrap_label.set_halign(gtk::Align::Start);
        let bootstrap_entry = gtk::Entry::new();
        grid.attach_next_to(&bootstrap_label,
                            Some(&secret_label),
                            gtk::PositionType::Bottom,
                            100, 12);
        grid.attach_next_to(&bootstrap_entry,
                            Some(&bootstrap_label),
                            gtk::PositionType::Right,
                            100, 12);

        let port_label = gtk::Label::new("Local Port");
        port_label.set_halign(gtk::Align::Start);
        let port_entry = gtk::Entry::new();
        grid.attach_next_to(&port_label,
                            Some(&bootstrap_label),
                            gtk::PositionType::Bottom,
                            100, 12);
        grid.attach_next_to(&port_entry,
                            Some(&port_label),
                            gtk::PositionType::Right,
                            100, 12);

        let connect_button = gtk::Button::new_with_label("Connect");

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.pack_start(&grid, false, false, 0);
        container.pack_start(&connect_button, false, false, 0);

        let c = configuration.clone();
        connect_button.connect_clicked(move |_| {
            c.borrow_mut().update(
                secret_entry.get_text(),
                bootstrap_entry.get_text(),
                port_entry.get_text().and_then(|port| u16::from_str(port.as_str()).ok())
            );
        });

        view.connect_delete_event(|_,_| {
            gtk::Inhibit(true)
        });

        view.add(&container);

        Rc::new(RefCell::new(Configuration {
            view: view
        }))
    }

    pub fn view(&self) -> &gtk::Window {
        &self.view
    }
}

pub struct ConversationRecipient {
    view: gtk::Entry,
    changed_signal: u64
}

impl ConversationRecipient {
    pub fn new(conversation: Rc<RefCell<models::Conversation>>) -> Rc<RefCell<ConversationRecipient>> {
        let view = gtk::Entry::new();

        match conversation.borrow().recipient() {
            Some(address) => view.set_text(&address.to_str()),
            None => view.set_text("New Conversation")
        }

        let c = conversation.clone();
        let changed_signal = view.connect_changed(move |entry| {
            let text = entry.get_text().unwrap();
            if text.len() == 40 {
                let address = address::Address::from_str(&text).ok();
                c.borrow_mut().set_recipient(address);
            }
        });

        let controller = Rc::new(RefCell::new(ConversationRecipient {
            view: view,
            changed_signal: changed_signal
        }));

        let observer_id = conversation.borrow_mut().register_observer(controller.clone());
        controller.borrow().view().connect_destroy(move |_| {
            conversation.borrow_mut().deregister_observer(&observer_id);
        });

        controller
    }

    pub fn view(&self) -> &gtk::Entry {
        &self.view
    }
}

impl ConversationObserver for ConversationRecipient {
    fn recipient_was_changed(&self, address: Option<comm::address::Address>) {
        signal::signal_handler_block(&self.view, self.changed_signal);
        match address {
            Some(a) => self.view.set_text(&a.to_str()),
            None => self.view.set_text("New Conversation")
        }
        signal::signal_handler_unblock(&self.view, self.changed_signal);
    }

    fn pending_message_was_changed(&self, _: String) { }
    fn did_receive_message(&self, _: Rc<RefCell<models::Message>>) { }
    fn did_send_message(&self, _: Rc<RefCell<models::Message>>) { }
}

pub struct MessageStatus {
    view: gtk::Label
}

impl MessageStatus {
    pub fn new(message: Rc<RefCell<models::Message>>) -> Rc<RefCell<MessageStatus>> {
        let view = gtk::Label::new(None);
        view.set_halign(gtk::Align::End);
        let style = view.get_style_context().unwrap();
        style.add_class("message__status");

        let controller = Rc::new(RefCell::new(MessageStatus {
            view: view
        }));

        message.borrow_mut().register_observer(controller.clone());

        controller
    }

    pub fn view(&self) -> &gtk::Label {
        &self.view
    }
}

impl MessageObserver for MessageStatus {
    fn did_receieve_acknowledgement(&self) {
        self.view().set_text("Acknowledged");
    }
}

pub struct Message {
    view: gtk::Box
}

impl Message {
    pub fn new(message: Rc<RefCell<models::Message>>) -> Rc<RefCell<Message>> {
        let view = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let style = view.get_style_context().unwrap();

        style.add_class("message");
        if message.borrow().was_sent() {
            view.set_halign(gtk::Align::End);
            style.add_class("message--sent");
        } else {
            view.set_halign(gtk::Align::Start);
            style.add_class("message--received");
        }

        let text = gtk::Label::new(Some(message.borrow().text()));
        text.set_line_wrap(true);
        let text_style = text.get_style_context().unwrap();
        text_style.add_class("message__text");
        text.set_halign(gtk::Align::Start);

        let status = MessageStatus::new(message.clone());

        view.pack_start(&text, false, false, 0);
        view.pack_start(status.borrow().view(), false, false, 0);

        let controller = Rc::new(RefCell::new(Message {
            view: view
        }));

        message.borrow_mut().register_observer(controller.clone());

        controller
    }

    pub fn view(&self) -> &gtk::Box {
        &self.view
    }
}

impl MessageObserver for Message {
    fn did_receieve_acknowledgement(&self) {
        let style = self.view().get_style_context().unwrap();
        style.add_class("message--acknowledged");
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

        let style = container.get_style_context().unwrap();

        style.add_class("transcript");

        viewport.add(&container);
        view.add(&viewport);

        let controller = Rc::new(RefCell::new(Transcript {
            view: view,
            container: container
        }));

        for message in conversation.borrow().messages().iter().cloned() {
            controller.borrow_mut().did_receive_message(message);
        }

        let observer_id = conversation.borrow_mut().register_observer(controller.clone());
        controller.borrow().view().connect_destroy(move |_| {
            conversation.borrow_mut().deregister_observer(&observer_id);
        });

        controller
    }

    pub fn view(&self) -> &gtk::ScrolledWindow {
        &self.view
    }

    fn scroll_to_bottom(&self) {
        let adj = self.view().get_vadjustment().unwrap();
        gtk::idle_add(move || {
            let new_value = adj.get_upper() - adj.get_page_size();
            adj.set_value(new_value);
            glib::Continue(false)
        });
    }
}

impl ConversationObserver for Transcript {
    fn recipient_was_changed(&self, _: Option<comm::address::Address>) { }
    fn pending_message_was_changed(&self, _: String) { }

    fn did_receive_message(&self, message: Rc<RefCell<models::Message>>) {
        let message_controller = Message::new(message);
        self.container.pack_start(message_controller.borrow().view(), false, false, 0);
        self.view().show_all();
        self.scroll_to_bottom();
    }

    fn did_send_message(&self, message: Rc<RefCell<models::Message>>) {
        let message_controller = Message::new(message);
        self.container.pack_start(message_controller.borrow().view(), false, false, 0);
        self.view().show_all();
        self.scroll_to_bottom();
    }
}

pub struct MessageEntry {
    view: gtk::Entry,
    changed_signal: u64
}

impl MessageEntry {
    pub fn new(conversation: Rc<RefCell<models::Conversation>>) -> Rc<RefCell<MessageEntry>> {
        let view = gtk::Entry::new();

        view.set_text(conversation.borrow().pending_message());

        let c = conversation.clone();
        let changed_signal = view.connect_changed(move |entry| {
            let text = entry.get_text().unwrap();
            c.borrow_mut().set_pending_message(text);
        });

        let c = conversation.clone();
        view.connect_key_press_event(move |_, event| {
            match event.get_keyval() {
                65293 => {
                    c.borrow_mut().send_message();
                    gtk::Inhibit(true)
                }
                _ => {
                    gtk::Inhibit(false)
                }
            }
        });

        let controller = Rc::new(RefCell::new(MessageEntry {
            view: view,
            changed_signal: changed_signal
        }));

        let observer_id = conversation.borrow_mut().register_observer(controller.clone());
        controller.borrow().view().connect_destroy(move |_| {
            conversation.borrow_mut().deregister_observer(&observer_id);
        });

        controller
    }

    pub fn view(&self) -> &gtk::Entry {
        &self.view
    }
}

impl ConversationObserver for MessageEntry {
    fn recipient_was_changed(&self, _: Option<comm::address::Address>) { }

    fn pending_message_was_changed(&self, pending_message: String) {
        signal::signal_handler_block(&self.view, self.changed_signal);
        self.view.set_text(&pending_message);
        signal::signal_handler_unblock(&self.view, self.changed_signal);
    }

    fn did_receive_message(&self, _: Rc<RefCell<models::Message>>) { }
    fn did_send_message(&self, _: Rc<RefCell<models::Message>>) { }
}

pub struct Conversation {
    view: gtk::Box
}

impl Conversation {
    pub fn new(conversation: Rc<RefCell<models::Conversation>>) -> Rc<RefCell<Conversation>> {
        let view = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let recipient_controller = ConversationRecipient::new(conversation.clone());
        let transcript_controller = Transcript::new(conversation.clone());
        let message_entry = MessageEntry::new(conversation.clone());

        view.pack_start(recipient_controller.borrow().view(), false, false, 0);
        view.pack_start(transcript_controller.borrow().view(), true, true, 0);
        view.pack_start(message_entry.borrow().view(), false, false, 0);

        let controller = Rc::new(RefCell::new(Conversation {
            view: view
        }));

        let observer_id = conversation.borrow_mut().register_observer(controller.clone());
        controller.borrow().view().connect_destroy(move |_| {
            conversation.borrow_mut().deregister_observer(&observer_id);
        });

        controller
    }

    pub fn view(&self) -> &gtk::Box {
        &self.view
    }
}

impl ConversationObserver for Conversation {
    fn recipient_was_changed(&self, _: Option<comm::address::Address>) { }
    fn pending_message_was_changed(&self, _: String) { }
    fn did_receive_message(&self, _: Rc<RefCell<models::Message>>) { }
    fn did_send_message(&self, _: Rc<RefCell<models::Message>>) { }
}

pub struct ConversationListItemTitle {
    view: gtk::Label
}

impl ConversationListItemTitle {
    pub fn new(conversation: Rc<RefCell<models::Conversation>>) -> Rc<RefCell<ConversationListItemTitle>> {
        let view = gtk::Label::new("");

        match conversation.borrow().recipient() {
            None => view.set_text("New Conversation"),
            Some(address) => view.set_text(&address.to_str())
        };

        let controller = Rc::new(RefCell::new(ConversationListItemTitle {
            view: view
        }));

        conversation.borrow_mut().register_observer(controller.clone());

        controller
    }

    pub fn view(&self) -> &gtk::Label {
        &self.view
    }
}
impl ConversationObserver for ConversationListItemTitle {
    fn recipient_was_changed(&self, address: Option<comm::address::Address>) {
        match address {
            Some(a) => self.view.set_text(&a.to_str()),
            None => self.view.set_text("New Conversation")
        }
    }

    fn pending_message_was_changed(&self, _: String) { }
    fn did_receive_message(&self, _: Rc<RefCell<models::Message>>) { }
    fn did_send_message(&self, _: Rc<RefCell<models::Message>>) { }
}

pub struct ConversationListItem {
    view: gtk::ListBoxRow
}

impl ConversationObserver for ConversationListItem {
    fn recipient_was_changed(&self, _: Option<comm::address::Address>) { }
    fn pending_message_was_changed(&self, _: String) { }
    fn did_receive_message(&self, _: Rc<RefCell<models::Message>>) { }
    fn did_send_message(&self, _: Rc<RefCell<models::Message>>) { }
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

        let observer_id = conversation.borrow_mut().register_observer(controller.clone());
        controller.borrow().view().connect_destroy(move |_| {
            conversation.borrow_mut().deregister_observer(&observer_id);
        });

        controller
    }

    pub fn view(&self) -> &gtk::ListBoxRow {
        &self.view
    }
}

pub struct ConversationList {
    view: gtk::ListBox
}

impl ConversationList {
    pub fn new(conversations: Rc<RefCell<models::ConversationList>>) -> Rc<RefCell<ConversationList>> {
        let view = gtk::ListBox::new();

        let cl = conversations.clone();
        view.connect_row_selected(move |_, list_item| {
            let index = list_item.as_ref().unwrap().get_index() as usize;
            cl.borrow().select_conversation(index);
        });

        let controller = Rc::new(RefCell::new(ConversationList {
            view: view
        }));

        let observer_id = conversations.borrow_mut().register_observer(controller.clone());
        controller.borrow().view().connect_destroy(move |_| {
            conversations.borrow_mut().deregister_observer(&observer_id);
        });

        controller
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

    fn conversation_was_selected(&self, _: Rc<RefCell<models::Conversation>>) { }
}

pub struct Conversations {
    view: gtk::Paned
}

impl Conversations {
    pub fn new(connection: Rc<RefCell<models::Connection>>, conversations: Rc<RefCell<models::ConversationList>>) -> Rc<RefCell<Conversations>> {
        let view = gtk::Paned::new(gtk::Orientation::Horizontal);

        view.set_position(300);

        let sidebar_pane = gtk::Paned::new(gtk::Orientation::Vertical);
        view.add1(&sidebar_pane);
        let search_add_pane = gtk::Paned::new(gtk::Orientation::Horizontal);

        let search = gtk::SearchEntry::new();
        let new_conversation_button = gtk::Button::new_from_icon_name("contact-new", 2);

        search_add_pane.pack1(&search, true, true);
        search_add_pane.pack2(&new_conversation_button, false, false);

        let conversation_list_controller = ConversationList::new(conversations.clone());

        sidebar_pane.pack1(&search_add_pane, false, false);
        sidebar_pane.add2(conversation_list_controller.borrow().view());

        let c = conversations.clone();
        new_conversation_button.connect_clicked(move |_| {
            let conversation = Rc::new(RefCell::new(models::Conversation::new(connection.clone())));
            c.borrow_mut().add_conversation(conversation);
            c.borrow_mut().select_conversation(0);
        });

        let controller = Rc::new(RefCell::new(Conversations {
            view: view
        }));

        let observer_id = conversations.borrow_mut().register_observer(controller.clone());
        controller.borrow().view().connect_destroy(move |_| {
            conversations.borrow_mut().deregister_observer(&observer_id);
        });

        controller
    }

    pub fn view(&self) -> &gtk::Paned {
        &self.view
    }
}

impl ConversationListObserver for Conversations {
    fn conversation_was_added(&self, _: Rc<RefCell<models::Conversation>>) { }

    fn conversation_was_selected(&self, conversation: Rc<RefCell<models::Conversation>>) {
        let conversation_controller = Conversation::new(conversation);
        if let Some(widget) = self.view.get_child2() {
            widget.destroy();
        }
        self.view.add2(conversation_controller.borrow().view());
        self.view.show_all();
    }
}
