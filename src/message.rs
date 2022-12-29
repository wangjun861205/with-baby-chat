use crate::websocket::WS;
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
    In { to: String, content: String },
    Out { from: String, content: String },
    Broadcast(String),
    Users(Vec<String>),
    Notify { level: NotifyLevel, content: String },
}

#[derive(Debug)]
pub enum InnerMessage {
    Register { name: String, addr: Addr<WS> },
    Deregister { name: String },
    Send { from: String, to: String, content: String },
    Users(Vec<String>),
    Broadcast { from: String, content: String },
    Out { from: String, content: String },
    Notify { level: NotifyLevel, content: String },
}

impl actix::Message for InnerMessage {
    type Result = ();
}
