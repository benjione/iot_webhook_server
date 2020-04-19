//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

use actix::prelude::*;
use std::collections::HashMap;

use serde::Deserialize;

use diesel::r2d2;
use diesel::SqliteConnection;

use crate::models::device;
use crate::models::user;

type Pool = r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>;

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(String)]
pub struct Connect {
    pub addr: Recipient<Message>,
    pub email: String,
    pub password: String,
    pub object: String,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
}

/// Session is disconnected
#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct SendWebhook {
    pub id: String,
    pub message: String,
}

/// `ChatServer` manages chat rooms and responsible for coordinating chat
/// session. implementation is super primitive
pub struct ChatServer {
    sessions: HashMap<String, Recipient<Message>>, // id -> addr
    db: Pool,
}

impl ChatServer {
    /// Send message to all users in the room
    fn send_message(&self, id: String, message: &str) {
        if let Some(addr) = self.sessions.get(&id) {
            let _ = addr.do_send(Message(message.to_owned()));
        }
    }

    pub fn new(db: Pool) -> Self {
        ChatServer {
            sessions: HashMap::new(),
            db: db,
        }
    }
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
    type Result = String;
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let user = user::User::get_user_from_email(&msg.email, &self.db);
        let user = match user {
            Some(user) => user,
            _ => return "none".to_string(),
        };

        let object = device::Device::get_device_and_register(user.id, msg.object, &self.db);
        let object = match object {
            Some(object) => object,
            _ => return "none".to_string(),
        };

        self.sessions
            .insert(object.registration_id.clone(), msg.addr);
        println!("Someone joined");

        return object.registration_id;
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        // remove address
        self.sessions.remove(&msg.id);

        device::Device::go_offline(&msg.id, &self.db)
    }
}

impl Handler<SendWebhook> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: SendWebhook, _: &mut Context<Self>) {
        self.send_message(msg.id, &msg.message)
    }
}
