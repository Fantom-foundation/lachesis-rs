use lachesis_rs::Server;

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let sys = actix::System::new("heartbeat-example");

    Server::start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
