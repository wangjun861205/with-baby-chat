use crate::message::{InnerMessage, NotifyLevel};
use crate::websocket::WS;
use crate::Author;
use actix::{Actor, Addr, AsyncContext, Context, Handler, WrapFuture};
use std::collections::HashMap;

pub struct Dispatcher<A: Author + Unpin + 'static> {
    addrs: HashMap<String, Addr<WS<A>>>,
}

impl<A: Author + Unpin + 'static> Dispatcher<A> {
    pub fn new() -> Self {
        Self { addrs: HashMap::new() }
    }
}

impl<A: Author + Unpin + 'static> Actor for Dispatcher<A> {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(1 << 32 - 1);
    }
}

impl<A: Author + Unpin + 'static> Handler<InnerMessage<A>> for Dispatcher<A> {
    type Result = ();
    fn handle(&mut self, msg: InnerMessage<A>, ctx: &mut Self::Context) -> Self::Result {
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
