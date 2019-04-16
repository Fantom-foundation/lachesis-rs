use lachesis_rs::Server;

/**
 * Main lachesis-rs entrypoint. Starts HTTP server.
 */
fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let sys = actix::System::new("heartbeat-example");

    let host = "127.0.0.1:8080";
    Server::init().bind(host).unwrap().start();

    println!("Started http server: {}", host);
    let _ = sys.run();
}
