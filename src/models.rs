use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;

use comm::address::Address;
use comm;

pub struct ObserverSet<O> {
    next_id: usize,
    observers: HashMap<usize, O>
}

impl<O> ObserverSet<O> {
    fn new() -> ObserverSet<O> {
        ObserverSet {
            next_id: 0,
            observers: HashMap::new()
        }
    }

    fn insert(&mut self, observer: O) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.observers.insert(id, observer);
        id
    }

    fn notify<F: Fn(&O)>(&self, function: F) {
        for (_, observer) in self.observers.iter() {
            function(observer);
        }
    }

    fn remove(&mut self, id: &usize) {
        self.observers.remove(id);
    }
}

pub trait Observable<O> {
    fn observers(&mut self) -> &mut ObserverSet<O>;

    fn register_observer(&mut self, observer: O) -> usize {
        self.observers().insert(observer)
    }

    fn deregister_observer(&mut self, id: &usize) {
        self.observers().remove(id);
    }
}

pub trait ConversationListObserver {
    fn conversation_was_added(&self, Rc<RefCell<Conversation>>);
    fn conversation_was_selected(&self, Rc<RefCell<Conversation>>);
}

pub trait ConversationObserver {
    fn recipient_was_changed(&self, Option<Address>);
    fn pending_message_was_changed(&self, String);
    fn did_receive_message(&mut self, Rc<RefCell<Message>>);
    fn did_send_message(&mut self, Rc<RefCell<Message>>);
}

pub trait MessageObserver {
    fn did_receieve_acknowledgement(&self);
}

pub struct Configuration {
    secret: Option<String>,
    router: Option<String>,
    port: Option<u16>
}

impl Configuration {
    pub fn empty() -> Configuration {
        Configuration {
            secret: None,
            router: None,
            port: None
        }
    }

    pub fn update(&mut self, secret: Option<String>, router: Option<String>, port: Option<u16>) {
        self.secret = secret;
        self.router = router;
        self.port = port;
    }

    fn secret(&self) -> &Option<String> {
        &self.secret
    }

    fn router(&self) -> &Option<String> {
        &self.router
    }

    fn port(&self) -> &Option<u16> {
        &self.port
    }
}

#[derive(Clone, Copy)]
pub enum ConnectionState {
    Running,
    Starting,
    Stopped,
    Stopping
}

pub trait ConnectionObserver {
    fn connection_started(&mut self, connection: &Connection);
    fn connection_shutdown(&mut self, connection: &Connection);
}

pub struct Connection {
    event_sender: mpsc::Sender<comm::client::Event>,
    commands: Option<comm::client::TaskSender>,
    self_address: Option<comm::address::Address>,
    state: ConnectionState,
    observers: ObserverSet<Rc<RefCell<ConnectionObserver>>>
}

impl Connection {
    pub fn new() -> (Rc<RefCell<Connection>>, comm::client::Events) {

        let (event_sender, events) = mpsc::channel();

        let connection = Rc::new(RefCell::new(Connection {
            event_sender: event_sender,
            commands: None,
            self_address: None,
            state: ConnectionState::Stopped,
            observers: ObserverSet::new()
        }));

        (connection, events)
    }

    pub fn commands(&self) -> comm::client::TaskSender {
        self.commands.as_ref().unwrap().clone()
    }

    pub fn self_address(&self) -> Address {
        self.self_address.unwrap()
    }

    pub fn shutdown(&mut self) {
        if let Some(c) = self.commands.as_ref() {
            c.send(comm::client::Task::Shutdown).expect("Failed to send Shutdown");
            self.state = ConnectionState::Stopping;
        }
    }

    pub fn start(&mut self, configuration: Ref<Configuration>) {
        self.self_address = configuration.secret().as_ref().map(|ref s| comm::address::Address::for_content(s.as_str()));

        let host = ("0.0.0.0", configuration.port().unwrap());

        let routers: Vec<comm::node::Node> = match configuration.router().as_ref() {
            Some(r) => {
                let router_node = comm::node::Node::new(comm::address::Address::null(), r.as_str());
                vec![router_node]
            }
            None => vec![]
        };

        let network = comm::network::Network::new(self.self_address.unwrap(), host, routers);
        let mut client = comm::client::Client::new(self.self_address.unwrap());
        client.register_event_listener(self.event_sender.clone());
        self.commands = Some(client.run(network));
        self.state = ConnectionState::Starting;
    }

    pub fn state(&self) -> ConnectionState {
        self.state
    }

    fn handle_shutdown(&mut self) {
        self.state = ConnectionState::Stopped;
        self.commands = None;
        self.self_address = None;
        self.observers.notify(|observer| {
            observer.borrow_mut().connection_shutdown(&self);
        });
    }

    fn handle_started(&mut self) {
        self.state = ConnectionState::Running;
        self.observers.notify(|observer| {
            observer.borrow_mut().connection_started(&self);
        });
    }
}

impl Observable<Rc<RefCell<ConnectionObserver>>> for Connection {
    fn observers(&mut self) -> &mut ObserverSet<Rc<RefCell<ConnectionObserver>>> {
        &mut self.observers
    }
}

#[derive(PartialEq, Eq)]
pub enum MessageDirection {
    Sent, Received
}

pub struct Message {
    id: Address,
    text: String,
    direction: MessageDirection,
    acknowledged: bool,
    observers: ObserverSet<Rc<RefCell<MessageObserver>>>
}

impl Message {
    pub fn new(id: Address, text: String, direction: MessageDirection) -> Message {
        Message {
            id: id,
            text: text,
            direction: direction,
            acknowledged: false,
            observers: ObserverSet::new()
        }

    }
    pub fn sent(id: Address, text: String) -> Message {
        Self::new(id, text, MessageDirection::Sent)
    }

    pub fn received(id: Address, text: String) -> Message {
        Self::new(id, text, MessageDirection::Received)
    }

    pub fn acknowledged(&self) -> bool {
        self.acknowledged
    }

    pub fn text(&self) -> &str{
        &self.text
    }

    pub fn was_sent(&self) -> bool {
        self.direction == MessageDirection::Sent
    }

    pub fn was_received(&self) -> bool {
        self.direction == MessageDirection::Received
    }

    fn receive_acknowledgement(&mut self) {
        self.acknowledged = true;
        self.observers.notify(|observer| {
            observer.borrow().did_receieve_acknowledgement();
        });
    }
}

impl Observable<Rc<RefCell<MessageObserver>>> for Message {
    fn observers(&mut self) -> &mut ObserverSet<Rc<RefCell<MessageObserver>>> {
        &mut self.observers
    }
}

pub struct Conversation {
    connection: Rc<RefCell<Connection>>,
    recipient: Option<Address>,
    pending_message: String,
    messages: Vec<Rc<RefCell<Message>>>,
    observers: ObserverSet<Rc<RefCell<ConversationObserver>>>
}

impl Conversation {
    pub fn new(connection: Rc<RefCell<Connection>>) -> Conversation {
        Conversation {
            connection: connection,
            recipient: None,
            pending_message: String::new(),
            messages: vec![],
            observers: ObserverSet::new()
        }
    }

    pub fn has_started(&self) -> bool {
        !self.messages.is_empty()
    }

    pub fn messages(&self) -> &Vec<Rc<RefCell<Message>>> {
        &self.messages
    }

    pub fn pending_message(&self) -> &str {
        &self.pending_message
    }

    pub fn set_pending_message(&mut self, text: String) {
        self.pending_message = text.clone();
        self.observers.notify(|observer| {
            observer.borrow().pending_message_was_changed(text.clone());
        });
    }

    pub fn set_recipient(&mut self, recipient: Option<Address>) {
        self.recipient = recipient;
        self.observers.notify(|observer| {
            observer.borrow().recipient_was_changed(recipient);
        });
    }

    pub fn receive_message(&mut self, message: Rc<RefCell<Message>>) {
        self.messages.push(message.clone());
        self.observers.notify(|observer| {
            observer.borrow_mut().did_receive_message(message.clone());
        })
    }

    pub fn recipient(&self) -> Option<Address> {
        self.recipient
    }

    pub fn send_message(&mut self) {
        if let Some(recipient) = self.recipient {
            let tm = comm::client::messages::TextMessage::new(
                self.connection.borrow().self_address(), self.pending_message.clone());

            self.connection.borrow().commands()
                .send(comm::client::Task::ScheduleMessageDelivery(recipient, tm.clone()))
                .expect("Couldn't send message");

            self.set_pending_message(String::new());

            let message = Rc::new(RefCell::new(Message::sent(tm.id, tm.text)));
            self.messages.push(message.clone());

            self.observers.notify(|observer| {
                observer.borrow_mut().did_send_message(message.clone());
            });
        }
    }
}

impl Observable<Rc<RefCell<ConversationObserver>>> for Conversation {
    fn observers(&mut self) -> &mut ObserverSet<Rc<RefCell<ConversationObserver>>> {
        &mut self.observers
    }
}

pub struct ConversationList {
    connection: Rc<RefCell<Connection>>,
    conversations: Vec<Rc<RefCell<Conversation>>>,
    observers: ObserverSet<Rc<RefCell<ConversationListObserver>>>
}

impl ConversationList {
    pub fn new(connection: Rc<RefCell<Connection>>) -> ConversationList {
        ConversationList {
            connection: connection,
            conversations: vec![],
            observers: ObserverSet::new()
        }
    }

    pub fn add_conversation(&mut self, conversation: Rc<RefCell<Conversation>>) {
        self.conversations.insert(0, conversation.clone());
        self.observers.notify(|observer| {
            observer.borrow().conversation_was_added(conversation.clone());
        });
    }

    pub fn get(&self, index: usize) -> Option<&Rc<RefCell<Conversation>>> {
        self.conversations.get(index)
    }

    pub fn select_conversation(&mut self, index: usize) {
        let conversation = self.get(index).unwrap().clone();
        self.observers.notify(|observer| {
            observer.borrow().conversation_was_selected(conversation.clone());
        });
    }

    pub fn handle_event(&mut self, event: comm::client::Event) {
        match event {
            comm::client::Event::ReceivedTextMessage(tm) => {
                let sender = tm.sender;
                let message = Rc::new(RefCell::new(Message::received(tm.id, tm.text)));
                let existing_conversation = self.conversations.iter().any(|conversation| {
                    conversation.borrow().recipient() == Some(sender)
                });

                if existing_conversation {
                    let c = self.conversations.iter().find(|conversation| {
                        conversation.borrow().recipient() == Some(sender)
                    }).unwrap();
                    c.borrow_mut().receive_message(message);
                } else {
                    let c = Rc::new(RefCell::new(Conversation::new(self.connection.clone())));
                    c.borrow_mut().set_recipient(Some(sender));
                    self.add_conversation(c.clone());
                    c.borrow_mut().receive_message(message);
                }
            }

            comm::client::Event::ReceivedMessageAcknowledgement(ack) => {
                for conversation in self.conversations.iter() {
                    for message in conversation.borrow().messages.iter() {
                        if message.borrow().id == ack.message_id {
                            message.borrow_mut().receive_acknowledgement();
                        }
                    }
                }
            }

            comm::client::Event::SentTextMessage(_) => { }

            // This is totally out of place. The Connection aught to be in the event handler.
            comm::client::Event::Shutdown => {
                self.connection.borrow_mut().handle_shutdown();
            }

            // This is totally out of place. The Connection aught to be in the event handler.
            comm::client::Event::Started => {
                self.connection.borrow_mut().handle_started();
            }
        }
    }
}

impl Observable<Rc<RefCell<ConversationListObserver>>> for ConversationList {
    fn observers(&mut self) -> &mut ObserverSet<Rc<RefCell<ConversationListObserver>>> {
        &mut self.observers
    }
}

pub struct EventHandler {
    conversations: Rc<RefCell<ConversationList>>
}

impl EventHandler {
    pub fn new(conversations: Rc<RefCell<ConversationList>>) -> EventHandler {
        EventHandler {
            conversations: conversations
        }
    }

    pub fn handle_event(&self, event: comm::client::Event) {
        debug!("Received event: {:?}", event.clone());
        self.conversations.borrow_mut().handle_event(event.clone());
    }
}
