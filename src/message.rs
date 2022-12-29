use crate::{websocket::WS, Author};
use actix::Addr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum NotifyLevel {
    Notify,
    Warning,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OuterMessage {
    In { token: String, to: String, content: String },
    Out { from: String, content: String },
    Broadcast { token: String, content: String },
    Users(Vec<String>),
    Notify { level: NotifyLevel, content: String },
}

#[derive(Debug)]
pub enum InnerMessage<A: Author + Unpin + 'static> {
    Register { name: String, addr: Addr<WS<A>> },
    Deregister { name: String },
    Send { from: String, to: String, content: String },
    Users(Vec<String>),
    Broadcast { from: String, content: String },
    Out { from: String, content: String },
    Notify { level: NotifyLevel, content: String },
}

impl<A: Author + Unpin + 'static> actix::Message for InnerMessage<A> {
    type Result = ();
}
