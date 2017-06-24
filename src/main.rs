extern crate env_logger;
extern crate comm;
extern crate gtk;

use gtk::prelude::*;
use gtk::{Button, Orientation, Paned, SearchEntry, Window, WindowType};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::env;

mod models;
mod controllers;

fn start_client() -> (comm::client::TaskSender, mpsc::Receiver<comm::client::Event>) {
    use comm::*;

    let args: Vec<String> = env::args().collect();
    let secret = args[1].clone();

    let address = address::Address::for_content(secret.as_str());
    let host = args[2].as_str();

    let routers: Vec<Box<node::Node>> = match args.get(3) {
        Some(router_address) => {
            let router_node = Box::new(node::UdpNode::new(address::Address::null(), router_address.as_str()));
            vec![router_node]
        }
        None => vec![]
    };

    let network = network::Network::new(address, host, routers);
    let mut client = client::Client::new(address);
    let (event_sender, events) = mpsc::channel();
    client.register_event_listener(event_sender);
    let client_channel = client.run(network);

    (client_channel, events)
}

fn main() {
    env_logger::init().unwrap();
    let (client_commands, client_events) = start_client();

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

    let conversations_controller = controllers::ConversationList::new(conversations.clone());

    conversations_controller.borrow().view().connect_row_selected(move |_, list_item| {
        let index = list_item.as_ref().unwrap().get_index() as usize;

        let list = conversations.borrow();
        let conversation_controller = controllers::Conversation::new();
        conversation_controller.set_conversation(list.get(index).unwrap());
        if let Some(widget) = main_pane.get_child2() {
            widget.destroy();
        }
        main_pane.add2(conversation_controller.view());
        main_pane.show_all();
    });

    search_add_pane.pack1(&search, true, true);
    search_add_pane.pack2(&button, false, false);

    sidebar_pane.pack1(&search_add_pane, false, false);
    sidebar_pane.add2(conversations_controller.borrow().view());


    button.connect_clicked(move |_| {
        let conversation = Rc::new(RefCell::new(models::Conversation::new(client_commands.clone())));
        conversations_controller.borrow().add_conversation(conversation);
    });

    main_window.show_all();

    gtk::idle_add(move || {
        if let Ok(event) = client_events.try_recv() {
            println!("Event: {:?}", event);
        }
        gtk::Continue(true)
    });

    gtk::main();
}
