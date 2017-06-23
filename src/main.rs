extern crate env_logger;
extern crate comm;
extern crate gtk;

use gtk::prelude::*;
use gtk::{Button, Orientation, Paned, SearchEntry, Window, WindowType};
use std::cell::RefCell;
use std::rc::Rc;

mod models;
mod views;

fn main() {
    env_logger::init().unwrap();

    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let conversations = Rc::new(RefCell::new(models::ConversationList::new()));

    let main_window = Window::new(WindowType::Toplevel);
    main_window.set_title("Comm Messenger");
    main_window.set_default_size(600, 350);
    main_window.set_position(gtk::WindowPosition::Center);
    main_window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });


    let main_pane = Paned::new(Orientation::Horizontal);
    main_pane.set_position(200);
    let sidebar_pane = Paned::new(Orientation::Vertical);
    main_pane.add1(&sidebar_pane);
    let search_add_pane = Paned::new(Orientation::Horizontal);

    main_window.add(&main_pane);

    let search = SearchEntry::new();
    let button = Button::new_from_icon_name("contact-new", 2);

    let conversation_list = views::ConversationList::new(conversations.clone());

    conversation_list.widget().connect_row_selected(move |_, list_item| {
        let conversation_view = views::Conversation::new();
        let index = list_item.as_ref().unwrap().get_index();
        let list = conversations.borrow();
        conversation_view.set_conversation(list.get(index as usize).unwrap());
        if let Some(widget) = main_pane.get_child2() {
            widget.destroy();
        }
        main_pane.add2(conversation_view.widget());
        main_pane.show_all();
    });

    search_add_pane.pack1(&search, true, true);
    search_add_pane.pack2(&button, false, false);

    sidebar_pane.pack1(&search_add_pane, false, false);
    sidebar_pane.add2(conversation_list.widget());


    button.connect_clicked(move |_| {
        let conversation = Rc::new(RefCell::new(models::Conversation::new()));
        conversation_list.add_conversation(conversation);
    });

    main_window.show_all();

    gtk::main();
}
