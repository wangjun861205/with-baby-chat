use actix::{self, Actor, Addr, AsyncContext, Context, Handler, Registry, StreamHandler};
use actix_web::web::{self, get, Data, Path};
use actix_web::{http::StatusCode, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws::{self, Message, ProtocolError, WebsocketContext, WsResponseBuilder};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Deserialize)]
pub struct Send {
    to: String,
    content: String,
}

#[derive(Debug, Serialize)]
pub struct Msg {
    from: String,
    content: String,
}

impl actix::Message for Msg {
    type Result = ();
}

struct WS {
    addrs: Data<Mutex<HashMap<String, Addr<WS>>>>,
    name: String,
}

impl Actor for WS {
    type Context = WebsocketContext<Self>;
}

impl Handler<Msg> for WS {
    type Result = ();
    fn handle(&mut self, msg: Msg, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for WS {
    fn handle(&mut self, item: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        let item = item.unwrap();
        match item {
            Message::Text(s) => {
                let msg: Send = serde_json::from_str(&s).unwrap();
                self.addrs
                    .lock()
                    .unwrap()
                    .get(&(msg.to))
                    .unwrap()
                    .try_send(Msg {
                        from: self.name.clone(),
                        content: msg.content,
                    })
                    .unwrap();
            }
            Message::Ping(m) => ctx.pong(&m),
            Message::Close(_) => {
                for a in self.addrs.lock().unwrap().values() {
                    a.try_send(Msg {
                        from: self.name.clone(),
                        content: "bye".into(),
                    })
                    .unwrap();
                }
                self.addrs.lock().unwrap().remove(&self.name);
            }
            _ => {}
        }
    }
}

async fn login(
    addrs: Data<Mutex<HashMap<String, Addr<WS>>>>,
    name: Path<(String,)>,
    req: HttpRequest,
    stream: web::Payload,
) -> HttpResponse {
    let name = name.into_inner().0;
    let (addr, resp) = WsResponseBuilder::new(
        WS {
            addrs: addrs.clone(),
            name: name.clone(),
        },
        &req,
        stream,
    )
    .start_with_addr()
    .unwrap();
    addrs.lock().unwrap().insert(name.clone(), addr);
    resp
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let addrs: Data<Mutex<HashMap<String, Addr<WS>>>> = Data::new(Mutex::new(HashMap::new()));
    HttpServer::new(move || {
        App::new()
            .app_data(addrs.clone())
            .route("/login/{name}", get().to(login))
    })
    .bind("0.0.0.0:8000")
    .unwrap()
    .run()
    .await
}
