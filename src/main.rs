extern crate glib;
extern crate env_logger;
extern crate comm;
extern crate gtk;

use gtk::prelude::*;
use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

mod models;
mod controllers;

fn start_client() -> (Rc<models::Connection>, comm::client::Events) {
    let args: Vec<String> = env::args().collect();
    let secret = args[1].as_str();
    let host = args[2].as_str();
    let router = args.get(3);
    let (connection, events) = models::Connection::start(secret, host, router);

    (Rc::new(connection), events)
}

fn main() {
    env_logger::init().unwrap();
    let (connection, events) = start_client();

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

    let (tx, rx) = mpsc::channel();
    GLOBAL.with(move |global| {
        *global.borrow_mut() = Some((conversations, rx));
    });

    thread::spawn(move || {
        for event in events.iter() {
            tx.send(event).unwrap();
            glib::idle_add(handle_event);
        }
    });

    gtk::main();
}

fn handle_event() -> glib::Continue {
    GLOBAL.with(|global| {
        if let Some((ref conversations, ref events)) = *global.borrow() {
            if let Ok(event) = events.try_recv() {
                conversations.borrow_mut().handle_event(event);
            }
        }
    });
    glib::Continue(false)
}

thread_local!(
    pub static GLOBAL: RefCell<Option<(Rc<RefCell<models::ConversationList>>, comm::client::Events)>> = RefCell::new(None);
);
