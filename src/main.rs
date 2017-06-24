extern crate env_logger;
extern crate comm;
extern crate gtk;

use gtk::prelude::*;
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

    let main_window = gtk::Window::new(gtk::WindowType::Toplevel);
    main_window.set_title("Comm Messenger");
    main_window.set_default_size(600, 350);
    main_window.set_position(gtk::WindowPosition::Center);
    main_window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let conversations_controller = controllers::Conversations::new(conversations, client_commands);
    main_window.add(conversations_controller.borrow().view());

    main_window.show_all();

    gtk::idle_add(move || {
        if let Ok(event) = client_events.try_recv() {
            println!("Event: {:?}", event);
        }
        gtk::Continue(true)
    });

    gtk::main();
}
