#[macro_use]
extern crate log;
extern crate gdk;
extern crate glib;
extern crate env_logger;
extern crate comm;
extern crate gtk;

use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

mod models;
mod controllers;


fn main() {
    env_logger::init().unwrap();

    if gtk::init().is_err() {
        error!("Failed to initialize GTK.");
        return;
    }

    let main_window = gtk::Window::new(gtk::WindowType::Toplevel);
    main_window.set_title("Comm Messenger");
    main_window.set_default_size(700, 400);
    main_window.set_position(gtk::WindowPosition::Center);
    main_window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::Inhibit(true)
    });

    let configuration = Rc::new(RefCell::new(models::Configuration::empty()));
    let configuration_controller = controllers::Configuration::new(configuration.clone());

    let (connection, events) = models::Connection::new(configuration);
    let conversations = Rc::new(RefCell::new(models::ConversationList::new(connection.clone())));
    let conversations_controller = controllers::Conversations::new(connection.clone(), conversations.clone());
    main_window.add(conversations_controller.borrow().view());

    main_window.show_all();
    configuration_controller.borrow().view().show_all();

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

    let css_provider = gtk::CssProvider::new();
    let screen = gdk::Screen::get_default().expect("Couldn't get default screen");
    gtk::StyleContext::add_provider_for_screen(&screen,
                                               &css_provider,
                                               gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    match css_provider.load_from_path("resources/style.css") {
        Ok(_) => debug!("Loaded stylesheet"),
        Err(err) => warn!("Failed to load stylesheet: {}", err)
    }

    main_window.connect_key_press_event(move |_, event| {
        if event.get_keyval() == 65474 { // F5
            match css_provider.load_from_path("resources/style.css") {
                Ok(_) => debug!("Reloaded stylesheet"),
                Err(err) => warn!("Failed to load stylesheet: {}", err)
            }
        }
        gtk::Inhibit(false)
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
