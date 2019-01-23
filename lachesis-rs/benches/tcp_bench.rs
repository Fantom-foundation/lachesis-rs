//#[cfg(bench)]
//mod benches {

    extern crate lachesis_rs;

    use lachesis_rs::tcp_server::{TcpApp, TcpNode};
    use lachesis_rs::{Event, BTreeHashgraph};
    use std::env::args;
    use std::sync::Arc;

    const BASE_PORT: usize = 9000;
    const USAGE: &'static str = "Usage: tcp-client [number of nodes]";

    use test::Bencher;
    use rand::Rng;
    use std::mem::replace;

    fn test_hashgraph() -> BTreeHashgraph {
        let event1 = Event::new(vec![b"42".to_vec()], None, Vec::new());
        let hash1 = event1.hash().unwrap();
        let event2 = Event::new(vec![b"fish".to_vec()], None, vec![1]);
        let hash2 = event2.hash().unwrap();
        let event3 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash1.clone(), hash2.clone())),
            Vec::new(),
        );
        let hash3 = event3.hash().unwrap();
        let event4 = Event::new(vec![b"42".to_vec()], None, vec![1]);
        let hash4 = event4.hash().unwrap();
        let event5 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash3.clone(), hash4.clone())),
            Vec::new(),
        );
        let hash5 = event5.hash().unwrap();
        let event6 = Event::new(vec![b"42".to_vec()], None, vec![2]);
        let hash6 = event6.hash().unwrap();
        let event7 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash5.clone(), hash6.clone())),
            Vec::new(),
        );
        let hash7 = event7.hash().unwrap();
        let mut hashgraph = BTreeHashgraph::new();
        hashgraph.insert(hash1.clone(), event1.clone());
        hashgraph.insert(hash2.clone(), event2.clone());
        hashgraph.insert(hash3.clone(), event3.clone());
        hashgraph.insert(hash4.clone(), event4.clone());
        hashgraph.insert(hash5.clone(), event5.clone());
        hashgraph.insert(hash6.clone(), event6.clone());
        hashgraph.insert(hash7.clone(), event7.clone());
        hashgraph
    }


    #[bench]
    fn setup_random_hashmap(b: &mut Bencher) {
        let mut val : u32 = 0;
        let mut rng = rand::IsaacRng::new_unseeded();
        let mut map = std::collections::HashMap::new();

        b.iter(|| { map.insert(rng.gen::<u8>() as usize, val); val += 1; })
    }

    #[bench]
    fn tcp(b: &mut Bencher) -> Option<Summary> {
        env_logger::init();
        let args: Vec<String> = args().collect();
        if args.len() != 2 {
            panic!(USAGE);
        }
        let mut rng = ring::rand::SystemRandom::new();
        let n_nodes = args[1].parse::<usize>().unwrap();
        let mut nodes = Vec::with_capacity(n_nodes);
        for i in 0..n_nodes {
            let a = format!("0.0.0.0:{}", BASE_PORT + i);
            nodes.push(Arc::new(Box::new(TcpNode::new(&mut rng, a))));
        }
        for node in nodes.iter() {
            for peer in nodes.iter() {
                if peer.node.get_id() != node.node.get_id() {
                    node.node.add_node(peer.clone()).unwrap();

                }
            }
        }

        let mut handles = Vec::with_capacity(n_nodes * 2);
        for node in nodes.iter() {
            let app = TcpApp(node.clone());
            let (handle1, handle2) = app.run();
            handles.push(handle1);
            handles.push(handle2);
        }
        for handle in handles {
            handle.join().unwrap();
        }

        b.bench(|| {
            for node in nodes.iter() {
                for peer in nodes.iter() {
                    if peer.node.get_id() != node.node.get_id() {
                        node.get_sync(peer.node.get_id(), Some(&test_hashgraph()) )
                    }
                }
            }
        })


    }
//}
