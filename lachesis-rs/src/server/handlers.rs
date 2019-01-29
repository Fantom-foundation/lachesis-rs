use actix_web::{AsyncResponder, Error, HttpMessage, HttpRequest, HttpResponse};

use futures::{future::result, Future};

use super::AppState;

use super::heartbeat::GetHeartbeatCount;

use actix::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitTransaction {
    pub signature: String,
    pub payload: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckTransactionStatus {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Peer {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerList {
    peers: Vec<Peer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionStatus {
    Complete,
    Pending,
    Failed,
}

pub fn submit_transaction(
    req: &HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    req.json()
        .from_err()
        .and_then(|val: SubmitTransaction| {
            println!("model: {:?}", val);
            Ok(HttpResponse::Ok().json(val)) // <- send response
        })
        .responder()
}

pub fn heartbeat(req: &HttpRequest<AppState>) -> HttpResponse {
    println!("{:?}", req);

    *(req.state().counter.lock().unwrap()) += 1;

    let res = req.state().heartbeat_counter.send(GetHeartbeatCount);

    Arbiter::spawn(
        res.map(|res| match res {
            Ok(result) => println!("Got result: {}", result),
            Err(err) => println!("Got error: {}", err),
        })
        .map_err(|e| {
            println!("Actor is probably dead: {}", e);
        }),
    );

    HttpResponse::Ok().body(format!(
        "Num of requests: {}",
        req.state().counter.lock().unwrap()
    ))
}

pub fn check_transaction_status(
    req: &HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    //TODO: implement get transaction status from id
    let transaction_id = req.match_info().get("id").expect("no id provided");

    result(Ok(HttpResponse::Ok().json(TransactionStatus::Failed))).responder()
}

pub fn get_peers(req: &HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    //TODO: implement get list of peers
    let peers = vec![Peer {
        id: "wefwef".to_string(),
    }];

    result(Ok(HttpResponse::Ok().json(peers))).responder()
}
