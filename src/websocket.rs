use crate::message::{Command, Input, InputMessage, Login, LoginResponse, NotifyLevel, Output, OutputMessage, RepeatLoginWarning};
use crate::{Author, LoginRequest};
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

    async fn handle_login(self, phone: String, password: String) -> LoginResponse {
        match self.author.auth(phone.clone(), password).await {
            Err(e) => {
                return LoginResponse {
                    phone,
                    token: "".into(),
                    err: e.to_string(),
                }
            }
            Ok(token) => {
                if let Some(t) = token {
                    return LoginResponse { phone, token: t, err: "".into() };
                }
                return LoginResponse {
                    phone,
                    token: "".into(),
                    err: "invalid account".into(),
                };
            }
        }
    }
}

impl<A: Author + Clone + Unpin + 'static> Actor for WS<A> {
    type Context = WebsocketContext<Self>;
}

impl<A: Author + Clone + Unpin + 'static> StreamHandler<Result<Message, ProtocolError>> for WS<A> {
    fn handle(&mut self, item: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        let item = item.unwrap();
        match item {
            Message::Text(s) => {
                if let Ok(login) = serde_json::from_str::<Login>(&s) {
                    ctx.address().do_send(login);
                    return;
                }
                match serde_json::from_str::<InputMessage>(&s) {
                    Ok(msg) => match self.author.verify(msg.token) {
                        Ok(phone) => ctx.address().do_send(Command { from: phone, input: msg.input }),
                        Err(e) => ctx.text(
                            serde_json::to_string(&OutputMessage {
                                output: Output::Notify {
                                    level: NotifyLevel::Error,
                                    content: e.to_string(),
                                },
                            })
                            .unwrap(),
                        ),
                    },
                    Err(e) => ctx.text(
                        serde_json::to_string(&OutputMessage {
                            output: Output::Notify {
                                level: NotifyLevel::Error,
                                content: e.to_string(),
                            },
                        })
                        .unwrap(),
                    ),
                }
                let msg: InputMessage = serde_json::from_str(&s).unwrap();
                match msg.input {
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

impl<A: Author + Clone + Unpin + 'static> Handler<Command> for WS<A> {
    type Result = ();
    fn handle(&mut self, msg: Command, ctx: &mut Self::Context) -> Self::Result {
        match msg.input {
            Input::AddFriend { phone } => {}
            _ => {}
        }
    }
}

impl<A: Author + Clone + Unpin + 'static> Handler<Login> for WS<A> {
    type Result = ();
    fn handle(&mut self, msg: Login, ctx: &mut Self::Context) -> Self::Result {
        let addr = ctx.address();
        let h = self.clone().handle_login(msg.phone, msg.password);
        ctx.spawn(
            Box::pin(async move {
                let msg = h.await;
                addr.do_send(msg);
            })
            .into_actor(&self.clone()),
        );
    }
}

impl<A: Author + Clone + Unpin + 'static> Handler<LoginResponse> for WS<A> {
    type Result = ();
    fn handle(&mut self, msg: LoginResponse, ctx: &mut Self::Context) -> Self::Result {
        if msg.token != "" {
            let mut users = self.users.write().unwrap();
            if let Some(addr) = users.get_mut(&msg.phone).unwrap() {
                addr.do_send(RepeatLoginWarning);
            }
            users.insert(msg.phone.clone(), Some(ctx.address()));
        }
        ctx.text(serde_json::to_string(&msg).unwrap())
    }
}

impl<A: Author + Clone + Unpin + 'static> Handler<RepeatLoginWarning> for WS<A> {
    type Result = ();
    fn handle(&mut self, _: RepeatLoginWarning, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(
            serde_json::to_string(&OutputMessage {
                output: Output::Notify {
                    level: NotifyLevel::Warning,
                    content: "repeat login".into(),
                },
            })
            .unwrap(),
        )
    }
}
