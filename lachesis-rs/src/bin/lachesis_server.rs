#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate json;

use actix_web::{
    error, http, middleware, server, App, AsyncResponder, Error, HttpMessage, HttpRequest,
    HttpResponse, Json,
};

use lachesis_rs::HttpServer;

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("lachesis_server");
    HttpServer::start();
    let _ = sys.run();
}
