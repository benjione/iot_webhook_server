use actix::*;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::Deserialize;
use std::time::{Duration, Instant};

use crate::server;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Deserialize)]
struct RegisterWebSocket {
    username: String,
    password: String,
    object: String,
}

/// Entry point for our route
#[get("/ws")]
pub async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
) -> Result<HttpResponse, Error> {
    println!("Someone connected!");
    ws::start(
        DeviceSession {
            id: "".to_string(),
            hb: Instant::now(),
            user: None,
            object: None,
            password: None,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

struct DeviceSession {
    /// unique session id
    id: String,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    /// user
    user: Option<String>,
    /// object
    object: Option<String>,
    /// Password
    password: Option<String>,
    /// Chat server
    addr: Addr<server::ChatServer>,
}

impl Actor for DeviceSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Service started!");
        // we'll start heartbeat process on session start.
        self.hb(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        println!("Someone disconnected!");
        // notify chat server
        self.addr.do_send(server::Disconnect {
            id: self.id.clone(),
        });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<server::Message> for DeviceSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for DeviceSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let data: RegisterWebSocket = serde_json::from_str(&text).unwrap();
                self.user = Some(data.username);
                self.object = Some(data.object);
                self.password = Some(data.password);
                self.connect_to_server(ctx);
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

impl DeviceSession {
    fn connect_to_server(&self, ctx: &mut ws::WebsocketContext<Self>) -> () {
        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of DeviceSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.addr
            .send(server::Connect {
                addr: addr.recipient(),
                email: self.user.as_ref().unwrap().clone(),
                password: self.user.as_ref().unwrap().clone(),
                object: self.object.as_ref().unwrap().clone(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => {
                        if res == "none" {
                            ctx.stop()
                        } else {
                            act.id = res
                        }
                    }
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                act.addr.do_send(server::Disconnect { id: act.id.clone() });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}
