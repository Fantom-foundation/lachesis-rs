use actix_web::{
    error, http, middleware, server, App, AsyncResponder, Error, HttpMessage, HttpRequest,
    HttpResponse, Json,
};

use bytes::BytesMut;
use futures::{future::result, Future, Stream};
use json::JsonValue;
use serde_json::Value;

pub struct HttpServer;

impl HttpServer {
    pub fn create_app() -> App {
        App::new()
            .middleware(middleware::Logger::default())
            .resource("/transaction", |r| {
                r.method(http::Method::POST).a(submit_transaction)
            })
            .resource("/transaction/{id}", |r| {
                r.method(http::Method::GET).a(check_transaction_status)
            })
            .resource("/peer", |r| r.method(http::Method::GET).a(get_peers))
    }

    pub fn start() -> Result<&'static str, Error> {
        server::new(|| HttpServer::create_app())
            .bind("127.0.0.1:8080")?
            .shutdown_timeout(1)
            .start();
        Ok("Started http server: 127.0.0.1:8080")
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SubmitTransaction {
    signature: String,
    payload: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheckTransactionStatus {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Peer {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PeerList {
    peers: Vec<Peer>,
}

#[derive(Debug, Serialize, Deserialize)]
enum TransactionStatus {
    Complete,
    Pending,
    Failed,
}

pub fn submit_transaction(req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    req.json()
        .from_err() // convert all errors into `Error`
        .and_then(|val: SubmitTransaction| {
            println!("model: {:?}", val);
            Ok(HttpResponse::Ok().json(val)) // <- send response
        })
        .responder()
}

pub fn check_transaction_status(
    req: &HttpRequest,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    //TODO: implement get transaction status from id
    let transaction_id = req.match_info().get("id").expect("no id provided");

    result(Ok(HttpResponse::Ok().json(TransactionStatus::Failed))).responder()
}

pub fn get_peers(req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    //TODO: implement get list of peers
    let peers = vec![Peer {
        id: "wefwef".to_string(),
    }];

    result(Ok(HttpResponse::Ok().json(peers))).responder()
}

#[cfg(test)]
mod tests {

    use super::*;
    use actix_web::test::TestServer;

    #[test]
    fn test_submit_transaction() {
        let mut server = TestServer::with_factory(HttpServer::create_app);

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
        let mut server = TestServer::with_factory(HttpServer::create_app);

        let request = server.client(http::Method::GET, "/peer").finish().unwrap();

        let response = server.execute(request.send()).unwrap();
        assert!(response.status().is_success());
    }

    #[test]
    fn test_check_transaction_status() {
        let mut server = TestServer::with_factory(HttpServer::create_app);

        let request = server
            .client(http::Method::GET, "/transaction/0x81732be82h")
            .finish()
            .unwrap();

        let response = server.execute(request.send()).unwrap();
        println!("{:?}", response.body().wait());
        assert!(response.status().is_success());
    }

}
