extern crate json;
extern crate serde_derive;

use lachesis_rs::HttpServer;

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("lachesis_server");
    match HttpServer::start() {
        Ok(connection_message) => println!("{:?}", connection_message),
        Err(e) => panic!(e),
    }
    let _ = sys.run();
}
