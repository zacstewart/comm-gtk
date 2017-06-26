extern crate env_logger;
extern crate comm;
extern crate gtk;

use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::env;

mod models;
mod controllers;

fn start_client() -> Rc<RefCell<models::Connection>> {
    let args: Vec<String> = env::args().collect();
    let secret = args[1].as_str();
    let host = args[2].as_str();
    let router = args.get(3);


    Rc::new(RefCell::new(models::Connection::start(secret, host, router)))
}

fn main() {
    env_logger::init().unwrap();
    let connection = start_client();

    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let conversations = Rc::new(RefCell::new(models::ConversationList::new(connection.clone())));

    let main_window = gtk::Window::new(gtk::WindowType::Toplevel);
    main_window.set_title("Comm Messenger");
    main_window.set_default_size(600, 350);
    main_window.set_position(gtk::WindowPosition::Center);
    main_window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let conversations_controller = controllers::Conversations::new(connection.clone(), conversations.clone());
    main_window.add(conversations_controller.borrow().view());

    main_window.show_all();

    gtk::idle_add(move || {
        if let Ok(event) = connection.borrow().events().try_recv() {
            conversations.borrow_mut().handle_event(event);
        }
        gtk::Continue(true)
    });

    gtk::main();
}
