use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashMap;

/// Kademlia's bucket size (commonly 20 in papers); we use a smaller number for demo
const K: usize = 8;

/// A 160-bit identifier, like in Kademlia (commonly from SHA-1 space)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct NodeId([u8; 20]);

impl NodeId {
    /// Create a random 160-bit NodeId
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 20];
        rng.fill(&mut bytes);
        NodeId(bytes)
    }

    /// Compute XOR distance between two IDs as a big-endian integer in bytes
    fn xor_distance(&self, other: &NodeId) -> [u8; 20] {
        let mut out = [0u8; 20];
        for i in 0..20 {
            out[i] = self.0[i] ^ other.0[i];
        }
        out
    }
}

/// Compare two 160-bit distances (big-endian) for sorting
fn compare_distances(a: &[u8; 20], b: &[u8; 20]) -> Ordering {
    for i in 0..20 {
        if a[i] < b[i] {
            return Ordering::Less;
        } else if a[i] > b[i] {
            return Ordering::Greater;
        }
    }
    Ordering::Equal
}

/// A basic node in the DHT
#[derive(Debug)]
struct Node {
    id: NodeId,
    storage: HashMap<Vec<u8>, Vec<u8>>, // very simple key-value store
    peers: Vec<NodeId>,                  // simplified k-bucket: LRU list, max K
}

impl Node {
    /// Create a new node with a random ID
    fn new() -> Self {
        Self {
            id: NodeId::random(),
            storage: HashMap::new(),
            peers: Vec::new(),
        }
    }

    /// Update local peer list (LRU behavior, max K, no self)
    fn track_peer(&mut self, peer: &NodeId) {
        if *peer == self.id {
            return;
        }
        if let Some(pos) = self.peers.iter().position(|p| p == peer) {
            let existing = self.peers.remove(pos);
            self.peers.push(existing);
        } else {
            self.peers.push(*peer);
            if self.peers.len() > K {
                self.peers.remove(0);
            }
        }
    }

    /// RPC: ping - used to check liveness
    fn rpc_ping(&mut self, from: &NodeId) -> bool {
        self.track_peer(from);
        true
    }

    /// RPC: store - store a key/value locally
    fn rpc_store(&mut self, from: &NodeId, key: Vec<u8>, value: Vec<u8>) {
        self.track_peer(from);
        self.storage.insert(key, value);
    }

    /// RPC: find_value - try to get a value for a key
    fn rpc_find_value(&mut self, from: &NodeId, key: &[u8]) -> Option<Vec<u8>> {
        self.track_peer(from);
        self.storage.get(key).cloned()
    }

    /// RPC: find_node - return up to K known nodes closest to the target id
    fn rpc_find_node(&mut self, from: &NodeId, target: &NodeId) -> Vec<NodeId> {
        self.track_peer(from);
        let mut peers = self.peers.clone();
        peers.sort_by(|a, b| {
            let da = target.xor_distance(a);
            let db = target.xor_distance(b);
            compare_distances(&da, &db)
        });
        peers.truncate(K);
        peers
    }
}

/// An in-memory network that owns nodes and forwards RPC calls between them
struct Network {
    nodes: HashMap<NodeId, Node>,
}

impl Network {
    fn new() -> Self {
        Self { nodes: HashMap::new() }
    }

    /// Create and register a new node with a unique ID; returns its NodeId
    fn add_node(&mut self) -> NodeId {
        loop {
            let node = Node::new();
            if !self.nodes.contains_key(&node.id) {
                let id = node.id;
                self.nodes.insert(id, node);
                return id;
            }
        }
    }

    /// Helper to print a node's ID as hex
    fn id_hex(id: &NodeId) -> String {
        id.0.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// RPC forwarding: ping from one node to another
    fn ping(&mut self, from: &NodeId, to: &NodeId) -> Option<bool> {
        let target = self.nodes.get_mut(to)?;
        Some(target.rpc_ping(from))
    }

    /// RPC forwarding: store a key/value on a target node
    fn store(&mut self, from: &NodeId, to: &NodeId, key: Vec<u8>, value: Vec<u8>) -> Option<()> {
        let target = self.nodes.get_mut(to)?;
        target.rpc_store(from, key, value);
        Some(())
    }

    /// RPC forwarding: find_value on a target node
    fn find_value(&mut self, from: &NodeId, to: &NodeId, key: &[u8]) -> Option<Option<Vec<u8>>> {
        let target = self.nodes.get_mut(to)?;
        Some(target.rpc_find_value(from, key))
    }

    /// RPC forwarding: find_node on a target node
    fn find_node(&mut self, from: &NodeId, to: &NodeId, target_id: &NodeId) -> Option<Vec<NodeId>> {
        let target = self.nodes.get_mut(to)?;
        Some(target.rpc_find_node(from, target_id))
    }
}

fn main() {
    // Build a small in-memory network and add nodes
    let mut network = Network::new();
    let id0 = network.add_node();
    let id1 = network.add_node();
    let id2 = network.add_node();

    println!("Node 0: {}", Network::id_hex(&id0));
    println!("Node 1: {}", Network::id_hex(&id1));
    println!("Node 2: {}", Network::id_hex(&id2));

    // Bootstrap: let nodes learn about each other by contacting
    let _ = network.ping(&id1, &id0);
    let _ = network.ping(&id2, &id0);
    let _ = network.ping(&id2, &id1);

    // Inter-node store and lookup
    let key = b"hello".to_vec();
    let value = b"world".to_vec();
    network.store(&id1, &id0, key.clone(), value.clone());
    let got = network.find_value(&id2, &id0, &key).unwrap_or(None);
    println!(
        "Lookup on node 0 (asked by 2): {:?}",
        got.map(|v| String::from_utf8_lossy(&v).to_string())
    );

    // Ask node 0 for nodes closest to id2
    if let Some(closest) = network.find_node(&id1, &id0, &id2) {
        let list: Vec<String> = closest.iter().map(|nid| Network::id_hex(nid)).collect();
        println!("Closest (from node 0's view) to node2: {:?}", list);
    }
}
