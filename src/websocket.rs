use crate::dispatcher::Dispatcher;
use crate::message::{InnerMessage, NotifyLevel, OuterMessage};
use crate::Author;
use actix::{Actor, Addr, Handler, StreamHandler};
use actix_web::web::Data;
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};

pub struct WS<A: Author + Unpin + 'static> {
    pub name: String,
    pub dispatcher: Data<Addr<Dispatcher<A>>>,
    pub author: Data<A>,
}

impl<A: Author + Unpin + 'static> Actor for WS<A> {
    type Context = WebsocketContext<Self>;
}

impl<A: Author + Unpin + 'static> Handler<InnerMessage<A>> for WS<A> {
    type Result = ();
    fn handle(&mut self, msg: InnerMessage<A>, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            InnerMessage::Send { from, to, content } => {
                ctx.text(serde_json::to_string(&OuterMessage::Out { from, content }).unwrap());
            }
            InnerMessage::Users(users) => {
                ctx.text(serde_json::to_string(&OuterMessage::Users(users)).unwrap());
            }
            InnerMessage::Out { from, content } => ctx.text(serde_json::to_string(&OuterMessage::Out { from: from, content: content }).unwrap()),
            _ => {}
        }
    }
}

impl<A: Author + Unpin + 'static> StreamHandler<Result<Message, ProtocolError>> for WS<A> {
    fn handle(&mut self, item: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        let item = item.unwrap();
        match item {
            Message::Text(s) => {
                let msg: OuterMessage = serde_json::from_str(&s).unwrap();
                match msg {
                    OuterMessage::In { token, to, content } => {
                        if let Err(e) = self.author.verify(token) {
                            ctx.text(
                                serde_json::to_string(&OuterMessage::Notify {
                                    level: NotifyLevel::Error,
                                    content: e.to_string(),
                                })
                                .unwrap(),
                            );
                            return;
                        }
                        self.dispatcher
                            .try_send(InnerMessage::Send {
                                from: self.name.clone(),
                                to: to,
                                content: content,
                            })
                            .unwrap();
                    }
                    OuterMessage::Out { from, content } => ctx.text(serde_json::to_string(&OuterMessage::Out { from: from, content: content }).unwrap()),
                    OuterMessage::Broadcast { token, content } => {
                        if let Err(e) = self.author.verify(token) {
                            ctx.text(
                                serde_json::to_string(&OuterMessage::Notify {
                                    level: NotifyLevel::Error,
                                    content: e.to_string(),
                                })
                                .unwrap(),
                            );
                            return;
                        }
                        self.dispatcher
                            .try_send(InnerMessage::Broadcast {
                                from: self.name.clone(),
                                content: content,
                            })
                            .unwrap();
                    }
                    _ => {}
                }
            }
            Message::Ping(m) => ctx.pong(&m),
            Message::Close(_) => self.dispatcher.try_send(InnerMessage::Deregister { name: self.name.clone() }).unwrap(),
            _ => {}
        }
    }
}
