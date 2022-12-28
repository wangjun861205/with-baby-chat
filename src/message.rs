use crate::WS;
use actix::Addr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum OuterMessage {
    In { to: String, content: String },
    Out { from: String, content: String },
    Users(Vec<String>),
}

#[derive(Debug)]
pub enum InnerMessage {
    Register {
        name: String,
        addr: Addr<WS>,
    },
    Deregister {
        name: String,
    },
    Send {
        from: String,
        to: String,
        content: String,
    },
    Users(Vec<String>),
}

impl actix::Message for InnerMessage {
    type Result = ();
}
