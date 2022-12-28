mod dispatcher;
mod message;

use crate::dispatcher::Dispatcher;
use actix::{self, Actor, Addr, AsyncContext, Context, Handler, Registry, StreamHandler};
use actix_web::web::{self, get, Data, Path};
use actix_web::{http::StatusCode, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws::{self, Message, ProtocolError, WebsocketContext, WsResponseBuilder};
use message::{InnerMessage, OuterMessage};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize)]
pub struct Msg {
    from: String,
    content: String,
}

impl actix::Message for Msg {
    type Result = ();
}

pub struct WS {
    pub name: String,
    pub dispatcher: Data<Addr<Dispatcher>>,
}

impl Actor for WS {
    type Context = WebsocketContext<Self>;
}

impl Handler<Msg> for WS {
    type Result = ();
    fn handle(&mut self, msg: Msg, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

impl Handler<InnerMessage> for WS {
    type Result = ();
    fn handle(&mut self, msg: InnerMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            InnerMessage::Send { from, to, content } => {
                ctx.text(serde_json::to_string(&OuterMessage::Out { from, content }).unwrap());
            }
            InnerMessage::Users(users) => {
                ctx.text(serde_json::to_string(&OuterMessage::Users(users)).unwrap());
            }
            _ => {}
        }
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for WS {
    fn handle(&mut self, item: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        let item = item.unwrap();
        match item {
            Message::Text(s) => {
                let msg: OuterMessage = serde_json::from_str(&s).unwrap();
                match msg {
                    OuterMessage::In { to, content } => {
                        self.dispatcher
                            .try_send(InnerMessage::Send {
                                from: self.name.clone(),
                                to: to,
                                content: content,
                            })
                            .unwrap();
                    }
                    OuterMessage::Out { from, content } => ctx.text(
                        serde_json::to_string(&OuterMessage::Out {
                            from: from,
                            content: content,
                        })
                        .unwrap(),
                    ),
                    _ => {}
                }
            }
            Message::Ping(m) => ctx.pong(&m),
            Message::Close(_) => self
                .dispatcher
                .try_send(InnerMessage::Deregister {
                    name: self.name.clone(),
                })
                .unwrap(),
            _ => {}
        }
    }
}

async fn login(
    dispatcher: Data<Addr<Dispatcher>>,
    name: Path<(String,)>,
    req: HttpRequest,
    stream: web::Payload,
) -> HttpResponse {
    let name = name.into_inner().0;
    let (addr, resp) = WsResponseBuilder::new(
        WS {
            name: name.clone(),
            dispatcher: dispatcher.clone(),
        },
        &req,
        stream,
    )
    .start_with_addr()
    .unwrap();
    dispatcher
        .try_send(InnerMessage::Register {
            name: name.clone(),
            addr: addr,
        })
        .unwrap();
    resp
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let dispatcher = Dispatcher::new();
    let addr = dispatcher.start();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(addr.clone()))
            .route("/login/{name}", get().to(login))
    })
    .bind("0.0.0.0:8000")
    .unwrap()
    .run()
    .await
}
