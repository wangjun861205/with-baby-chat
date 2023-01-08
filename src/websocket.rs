use crate::message::{Command, Input, InputMessage, Login, LoginResponse, NotifyLevel, Output, OutputMessage, RepeatLoginWarning};
use crate::{Author, Dao};
use actix::{Actor, Addr, AsyncContext, Handler, StreamHandler, WrapFuture};
use actix_web::web::Data;
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Clone)]
pub struct WS<A, D>
where
    A: Author + Clone + Unpin + 'static,
    D: Dao + Clone + Unpin + 'static,
{
    pub name: String,
    pub author: Data<A>,
    pub users: Data<RwLock<HashMap<String, Option<Addr<WS<A, D>>>>>>,
    pub dao: Data<D>,
}

impl<A, D> WS<A, D>
where
    A: Author + Clone + Unpin + 'static,
    D: Dao + Clone + Unpin + 'static,
{
    pub fn new(name: String, author: Data<A>, users: Data<RwLock<HashMap<String, Option<Addr<WS<A, D>>>>>>, dao: Data<D>) -> Self {
        Self { name, author, users, dao }
    }

    async fn handle_login(self, phone: String, password: String) -> LoginResponse {
        match self.dao.get_account(phone.clone()).await {
            Err(e) => {
                return LoginResponse {
                    phone,
                    token: "".into(),
                    err: e.to_string(),
                }
            }
            Ok(acct) => {
                if let Some(a) = acct {
                    let hashed_pwd = self.author.hash_password(password, a.salt);
                    if hashed_pwd != a.password {
                        return LoginResponse {
                            phone,
                            token: "".into(),
                            err: "invalid phone or password".into(),
                        };
                    }
                    match self.dao.get_user_by_account_id(a.id).await {
                        Ok(user) => {
                            if let Some(u) = user {
                                match self.author.gen_token(u.id) {
                                    Ok(token) => return LoginResponse { phone, token: token, err: "".into() },
                                    Err(e) => {
                                        return LoginResponse {
                                            phone,
                                            token: "".into(),
                                            err: e.to_string(),
                                        }
                                    }
                                }
                            }
                            return LoginResponse {
                                phone,
                                token: "".into(),
                                err: "user not exists".into(),
                            };
                        }
                        Err(e) => {
                            return LoginResponse {
                                phone,
                                token: "".into(),
                                err: e.to_string(),
                            }
                        }
                    }
                }
                return LoginResponse {
                    phone,
                    token: "".into(),
                    err: "invalid phone or password".into(),
                };
            }
        }
    }
}

impl<A, D> Actor for WS<A, D>
where
    A: Author + Clone + Unpin + 'static,
    D: Dao + Clone + Unpin + 'static,
{
    type Context = WebsocketContext<Self>;
}

impl<A, D> StreamHandler<Result<Message, ProtocolError>> for WS<A, D>
where
    A: Author + Clone + Unpin + 'static,
    D: Dao + Clone + Unpin + 'static,
{
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
                        Ok(uid) => ctx.address().do_send(Command { from: uid, input: msg.input }),
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

impl<A, D> Handler<Command> for WS<A, D>
where
    A: Author + Clone + Unpin + 'static,
    D: Dao + Clone + Unpin + 'static,
{
    type Result = ();
    fn handle(&mut self, msg: Command, ctx: &mut Self::Context) -> Self::Result {
        match msg.input {
            Input::FindUser { phone } => {
                let addr = ctx.address();
                let dao = self.dao.clone();
                ctx.spawn(
                    async move {
                        match self.dao.get_account(phone).await {
                            Err(e) => addr.do_send(OutputMessage {
                                output: Output::Notify {
                                    level: NotifyLevel::Error,
                                    content: e.to_string(),
                                },
                            }),
                            Ok(user) => addr.do_send(OutputMessage {
                                output: Output::FindUserResponse { user },
                            }),
                        }
                    }
                    .into_actor(&self.clone()),
                )
            }
            _ => {}
        }
    }
}

impl<A, D> Handler<OutputMessage> for WS<A, D>
where
    A: Author + Clone + Unpin + 'static,
    D: Dao + Clone + Unpin + 'static,
{
    type Result = ();
    fn handle(&mut self, msg: OutputMessage, ctx: &mut Self::Context) -> Self::Result {}
}

impl<A, D> Handler<Login> for WS<A, D>
where
    A: Author + Clone + Unpin + 'static,
    D: Dao + Clone + Unpin + 'static,
{
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

impl<A, D> Handler<LoginResponse> for WS<A, D>
where
    A: Author + Clone + Unpin + 'static,
    D: Dao + Clone + Unpin + 'static,
{
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

impl<A, D> Handler<RepeatLoginWarning> for WS<A, D>
where
    A: Author + Clone + Unpin + 'static,
    D: Dao + Clone + Unpin + 'static,
{
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
