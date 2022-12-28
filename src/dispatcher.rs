use crate::{message::InnerMessage, WS};
use actix::{Actor, Addr, Context, Handler};
use std::collections::HashMap;

pub struct Dispatcher {
    addrs: HashMap<String, Addr<WS>>,
}

impl Actor for Dispatcher {
    type Context = Context<Self>;
}

impl Handler<InnerMessage> for Dispatcher {
    type Result = ();
    fn handle(&mut self, msg: InnerMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            InnerMessage::Register { name, addr } => {
                self.addrs.insert(name, addr);
            }
            InnerMessage::Send { from, to, content } => {
                self.addrs
                    .get(&(to))
                    .unwrap()
                    .try_send(InnerMessage::Send { from, to, content })
                    .unwrap();
            }
        }
    }
}
