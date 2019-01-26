use actix_web::{http, middleware, server, App};

use actix::prelude::*;

use std::sync::Arc;
use std::sync::Mutex;

pub mod handlers;
mod heartbeat;

use self::handlers::{check_transaction_status, get_peers, heartbeat, submit_transaction};
use self::heartbeat::Heartbeat;
pub struct Server;

#[derive(Clone)]
pub struct AppState {
    counter: Arc<Mutex<usize>>,
    heartbeat_counter: Addr<Heartbeat>,
}

impl Server {
    pub fn create_app() -> App<AppState> {
        let addr = Arbiter::start(move |_| Heartbeat { count: 0 });

        let counter = Arc::new(Mutex::new(0));

        App::with_state(AppState {
            counter: counter.clone(),
            heartbeat_counter: addr.clone(),
        })
        .middleware(middleware::Logger::default())
        .resource("/transaction", |r| {
            r.method(http::Method::POST).a(submit_transaction)
        })
        .resource("/transaction/{id}", |r| {
            r.method(http::Method::GET).a(check_transaction_status)
        })
        .resource("/peer", |r| r.method(http::Method::GET).f(get_peers))
        .resource("/heartbeat", |r| r.f(heartbeat))
    }

    pub fn start() {
        let counter = Arc::new(Mutex::new(0));

        let addr = Arbiter::start(move |_| Heartbeat { count: 0 });

        server::new(move || {
            App::with_state(AppState {
                counter: counter.clone(),
                heartbeat_counter: addr.clone(),
            })
            .middleware(middleware::Logger::default())
            .resource("/transaction", |r| {
                r.method(http::Method::POST).a(submit_transaction)
            })
            .resource("/transaction/{id}", |r| {
                r.method(http::Method::GET).a(check_transaction_status)
            })
            .resource("/peer", |r| r.method(http::Method::GET).f(get_peers))
            .resource("/heartbeat", |r| r.f(heartbeat))
        })
        .bind("127.0.0.1:8080")
        .unwrap()
        .start();
    }
}

#[cfg(test)]
mod tests {

    use super::handlers::SubmitTransaction;
    use super::*;
    use actix_web::test::TestServer;
    use actix_web::HttpMessage;
    use futures::future::Future;

    #[test]
    fn test_submit_transaction() {
        let mut server = TestServer::with_factory(Server::create_app);

        let request = server
            .client(http::Method::POST, "/transaction")
            .json(SubmitTransaction {
                signature: "efwef".to_string(),
                payload: "WEfwef".to_string(),
            })
            .unwrap();

        let response = server.execute(request.send()).unwrap();
        assert!(response.status().is_success());
    }

    #[test]
    fn test_get_peers() {
        let mut server = TestServer::with_factory(Server::create_app);

        let request = server.client(http::Method::GET, "/peer").finish().unwrap();

        let response = server.execute(request.send()).unwrap();
        assert!(response.status().is_success());
    }

    #[test]
    fn test_check_transaction_status() {
        let mut server = TestServer::with_factory(Server::create_app);

        let request = server
            .client(http::Method::GET, "/transaction/0x81732be82h")
            .finish()
            .unwrap();

        let response = server.execute(request.send()).unwrap();
        println!("{:?}", response.body().wait());
        assert!(response.status().is_success());
    }

}
