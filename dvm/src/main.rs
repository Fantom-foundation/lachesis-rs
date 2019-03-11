#[macro_use]
extern crate serde_derive;

use configure::Configure;
use failure::{Error, Fail};
use lachesis_rs::tcp_server::{TcpApp, TcpNode, TcpPeer};
use lachesis_rs::{BTreeHashgraph, Node, Swirlds};
use std::convert::TryFrom;
use std::io::Read;
use std::net::TcpListener;
use std::str::FromStr;
use std::sync::Arc;
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;
use vm::instruction::Program;
use vm::Cpu;

#[derive(Debug, Fail)]
enum DvmError {
    #[fail(display = "Wrong address {}", addr)]
    WrongAddressFormat { addr: String },
}

#[derive(Configure, Deserialize)]
#[serde(default)]
struct Config {
    cpu_memory: usize,
    lachesis_port: usize,
    peer_hosts: String,
    peer_ids: String,
    server_port: usize,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            cpu_memory: 1024,
            lachesis_port: 9000,
            peer_ids: String::from(""),
            peer_hosts: String::from(""),
            server_port: 8080,
        }
    }
}

struct Server {
    node: Arc<TcpNode<Swirlds<TcpPeer, BTreeHashgraph>>>,
    port: usize,
}

impl Server {
    fn new(port: usize, node: Arc<TcpNode<Swirlds<TcpPeer, BTreeHashgraph>>>) -> Server {
        Server { node, port }
    }

    fn run(self, cpu_memory: usize) -> (JoinHandle<()>, JoinHandle<()>) {
        let server = self.get_server_handle();
        let node = self.node.clone();
        let queue_consumer = spawn(move || {
            let next_to_process = 0;
            let mut cpu = Cpu::new(cpu_memory).unwrap();
            loop {
                let events = node.node.get_ordered_events().unwrap();
                let transactions: Vec<Vec<u8>> = events.iter().flat_map(|e| e.payload()).collect();
                if transactions.len() > next_to_process {
                    for i in next_to_process..transactions.len() - 1 {
                        let program = Program::try_from(transactions[i].clone()).unwrap();
                        cpu.execute(program).unwrap();
                    }
                }
                sleep(Duration::from_millis(100));
            }
        });
        (server, queue_consumer)
    }

    fn get_server_handle(&self) -> JoinHandle<()> {
        let port = self.port;
        let node = self.node.clone();
        spawn(move || {
            let address = format!("0.0.0.0:{}", port);
            let listener = TcpListener::bind(address).unwrap();
            for stream_result in listener.incoming() {
                let mut stream = stream_result.unwrap();
                let mut content = Vec::new();
                stream.read_to_end(&mut content).unwrap();
                node.node.add_transaction(content).unwrap();
            }
        })
    }
}

fn parse_peer(input: String) -> Result<(String, usize), Error> {
    let elements: Vec<String> = input.clone().split(':').map(|s| s.to_string()).collect();
    if elements.len() == 2 {
        Ok((elements[0].clone(), usize::from_str(&elements[1])?))
    } else {
        Err(Error::from(DvmError::WrongAddressFormat { addr: input }))
    }
}

fn parse_peers(input: String) -> Result<Vec<(String, usize)>, Error> {
    input
        .split(',')
        .map(|ps| parse_peer(ps.to_string()))
        .collect()
}

fn main() {
    env_logger::init();
    let config: Config = Config::generate().unwrap();
    let ids: Vec<String> = config.peer_ids.split(',').map(|s| s.to_string()).collect();
    let peers = parse_peers(config.peer_hosts).unwrap();
    if peers.len() != ids.len() {
        panic!("Number of peer ids mismatches number of peer addresses");
    }
    let peers: Vec<TcpPeer> = ids
        .iter()
        .zip(peers.iter())
        .map(|(id, (a, p))| TcpPeer {
            address: format!("{}:{}", a, p),
            id: id.as_bytes().to_vec(),
        })
        .collect();
    let mut rng = ring::rand::SystemRandom::new();
    let local_address = format!("0.0.0.0:{}", config.lachesis_port);
    let node = Arc::new(TcpNode::new(&mut rng, local_address).unwrap());
    for peer in peers.iter() {
        node.node.add_node(Arc::new(peer.clone())).unwrap();
    }
    let app = TcpApp::new(node.clone());
    let server = Server::new(config.server_port, node.clone());
    let (handle1, handle2) = app.run().unwrap();
    let (server_handle1, server_handle2) = server.run(config.cpu_memory);
    handle1.join().unwrap();
    handle2.join().unwrap();
    server_handle1.join().unwrap();
    server_handle2.join().unwrap();
}
