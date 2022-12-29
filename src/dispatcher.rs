use crate::message::InnerMessage;
use crate::websocket::WS;
use actix::{Actor, Addr, Context, Handler};
use std::collections::HashMap;

pub struct Dispatcher {
    addrs: HashMap<String, Addr<WS>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self { addrs: HashMap::new() }
    }
}

impl Actor for Dispatcher {
    type Context = Context<Self>;
}

impl Handler<InnerMessage> for Dispatcher {
    type Result = ();
    fn handle(&mut self, msg: InnerMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            InnerMessage::Register { name, addr } => {
                let users = self.addrs.keys().map(|v| v.clone()).collect::<Vec<String>>();
                addr.try_send(InnerMessage::Users(users)).unwrap();
                self.addrs.insert(name, addr);
            }
            InnerMessage::Deregister { name } => {
                self.addrs.remove(&name);
            }
            InnerMessage::Send { from, to, content } => {
                self.addrs.get(&(to)).unwrap().try_send(InnerMessage::Out { from, content }).unwrap();
            }
            InnerMessage::Broadcast { from, content } => {
                for addr in self.addrs.values() {
                    addr.try_send(InnerMessage::Out {
                        from: from.clone(),
                        content: content.clone(),
                    })
                    .unwrap();
                }
            }

            _ => {}
        }
    }
}
