use crate::message::{InnerMessage, NotifyLevel, OuterMessage};
use crate::Author;
use actix::{Actor, Addr, AsyncContext, Handler, StreamHandler, WrapFuture};
use actix_web::web::Data;
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use sha2::{Digest, Sha384};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct WS<A: Author + Clone + Unpin + 'static> {
    pub name: String,
    pub author: Data<A>,
    pub users: Data<RwLock<HashMap<String, Option<Addr<WS<A>>>>>>,
    pub anti_replay_token: String,
}

impl<A: Author + Clone + Unpin + 'static> WS<A> {
    pub fn new(name: String, author: Data<A>, users: Data<RwLock<HashMap<String, Option<Addr<WS<A>>>>>>) -> Self {
        Self {
            name,
            author,
            users,
            anti_replay_token: Uuid::new_v4().to_string(),
        }
    }

    fn verify_signature(&self, content: String, signature: String) -> bool {
        let mut hasher = Sha384::new();
        hasher.update(format!("{}{}", content, self.anti_replay_token.clone()));
        format!("{:x}", hasher.finalize()) == signature
    }

    fn refresh_anti_replay_token(&mut self) -> String {
        let token = Uuid::new_v4().to_string();
        self.anti_replay_token = token.clone();
        token
    }

    async fn handleLogin(self, username: String, password: String) -> InnerMessage<A> {
        match self.author.auth(username.clone(), password).await {
            Err(e) => {
                return InnerMessage::Notify {
                    level: NotifyLevel::Error,
                    content: e.to_string(),
                }
            }
            Ok(token) => {
                if let Some(t) = token {
                    return InnerMessage::LoginResponse { token: t };
                }
                return InnerMessage::Notify {
                    level: NotifyLevel::Warning,
                    content: "invalid account".into(),
                };
            }
        }
    }

    fn handleLoginResponse(&mut self, token: String) -> OuterMessage {
        let anti_replay_token = Uuid::new_v4().to_string();
        self.anti_replay_token = anti_replay_token.clone();
        OuterMessage::LoginResponse { token, anti_replay_token }
    }
}

impl<A: Author + Clone + Unpin + 'static> Actor for WS<A> {
    type Context = WebsocketContext<Self>;
}

impl<A: Author + Clone + Unpin + 'static> Handler<InnerMessage<A>> for WS<A> {
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
            InnerMessage::Login { username, password } => {
                let addr = ctx.address();
                let actor = self.clone();
                ctx.spawn(
                    async move {
                        let msg = actor.handleLogin(username, password).await;
                        addr.do_send(msg);
                    }
                    .into_actor(&self.clone()),
                );
            }
            InnerMessage::LoginResponse { token } => {
                let msg = self.handleLoginResponse(token);
                ctx.text(serde_json::to_string(&msg).unwrap());
            }
            _ => {}
        }
    }
}

impl<A: Author + Clone + Unpin + 'static> StreamHandler<Result<Message, ProtocolError>> for WS<A> {
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
                        if let Some(addr) = self.users.read().unwrap().get(&to) {
                            if let Some(a) = addr {
                                a.try_send(InnerMessage::Send {
                                    from: self.name.clone(),
                                    to: to,
                                    content: content,
                                })
                                .unwrap();
                            }
                            return;
                        }
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
                        for addr in self.users.read().unwrap().values() {
                            if let Some(a) = addr {
                                a.try_send(InnerMessage::Broadcast {
                                    from: self.name.clone(),
                                    content: content.clone(),
                                })
                                .unwrap();
                            }
                        }
                    }
                    OuterMessage::Login { username, password, signature } => {
                        if !self.verify_signature(format!("{}{}", username, password), signature) {
                            ctx.text(
                                serde_json::to_string(&OuterMessage::Notify {
                                    level: NotifyLevel::Error,
                                    content: "invalid signature".into(),
                                })
                                .unwrap(),
                            );
                            return;
                        }
                        if let Err(e) = ctx.address().try_send(InnerMessage::Login { username, password }) {
                            ctx.text(
                                serde_json::to_string(&OuterMessage::Notify {
                                    level: NotifyLevel::Error,
                                    content: e.to_string(),
                                })
                                .unwrap(),
                            )
                        }
                    }
                    OuterMessage::AntiReplayToken => ctx.text(
                        serde_json::to_string(&OuterMessage::AntiReplayTokenResponse {
                            token: self.anti_replay_token.clone(),
                        })
                        .unwrap(),
                    ),
                    _ => {}
                }
            }
            Message::Ping(m) => ctx.pong(&m),
            Message::Close(_) => {
                self.users.write().unwrap().remove(&self.name);
            }
            _ => {}
        }
    }
}
