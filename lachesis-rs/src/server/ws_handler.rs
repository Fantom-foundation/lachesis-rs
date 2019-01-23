use actix::prelude::*;
use actix_web::{ws, Error, HttpRequest, HttpResponse};
use bincode::deserialize;

use super::ws_message::InternodeMessage;
use super::AppState;

pub fn ws_index(r: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    info!("Websocket handshake");
    ws::start(r, Ws)
}

struct Ws;

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self, AppState>;
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Ws {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => {
                ctx.pong(&msg);
            }
            ws::Message::Pong(msg) => {
                ctx.ping(&msg);
            }
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(mut bin) => {
                let decoded: InternodeMessage = deserialize(&bin.take()).unwrap();
                info!("{:?}", decoded);
            }
            ws::Message::Close(_) => {
                ctx.stop();
            }
        }
    }
}
