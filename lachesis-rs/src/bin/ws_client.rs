use std::{io, thread};

use actix::*;

#[macro_use]
extern crate log;

use actix_web::ws::{Client, ClientWriter, Message, ProtocolError};
use futures::Future;

use lachesis_rs::InternodeMessage;

/**
 * Main lachesis-rs WebSocket client entrypoint. Starts client and connects to server.
 */
fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    let _ = env_logger::init();

    let sys = actix::System::new("ws-client");

    Arbiter::spawn(
        Client::new("http://127.0.0.1:8080/ws")
            .connect()
            .map_err(|e| {
                error!("Error: {}", e);
                ()
            })
            .map(|(reader, writer)| {
                let addr = InternodeClient::create(|ctx| {
                    InternodeClient::add_stream(reader, ctx);
                    InternodeClient(writer)
                });

                thread::spawn(move || loop {
                    let mut cmd = String::new();
                    if io::stdin().read_line(&mut cmd).is_err() {
                        error!("error");
                    }
                    addr.do_send(InternodeMessage::SyncRequest)
                });

                ()
            }),
    );

    let _ = sys.run();
}

struct InternodeClient(ClientWriter);

/**
 * Implement Actor for InternodeClient to start and stop
 */
impl Actor for InternodeClient {
    type Context = Context<Self>;

    /**
     * Start connection
     */
    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("Connected");
    }

    /**
     * Stop connection
     */
    fn stopped(&mut self, _: &mut Context<Self>) {
        info!("Disconnected");

        System::current().stop();
    }
}

impl Handler<InternodeMessage> for InternodeClient {
    type Result = ();

    fn handle(&mut self, msg: InternodeMessage, _ctx: &mut Context<Self>) {
        info!("Handling message");
        self.0.binary(msg)
    }
}

impl StreamHandler<Message, ProtocolError> for InternodeClient {
    fn handle(&mut self, msg: Message, _ctx: &mut Context<Self>) {
        match msg {
            Message::Binary(bin) => println!("Server: {:?}", bin),
            Message::Text(text) => println!("Server: {:?}", text),
            _ => (),
        }
    }

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("Connected");
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        info!("Server disconnected");
        ctx.stop()
    }
}
