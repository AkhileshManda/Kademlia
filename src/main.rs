use rand::Rng;
use sha1::{Digest, Sha1};
use std::cmp::Ordering;
use std::collections::HashMap;

/// Kademlia's bucket size (commonly 20 in papers); we use a smaller number for demo
const K: usize = 8;
/// Concurrency factor alpha in Kademlia (number of parallel queries); we serialize for simplicity
const ALPHA: usize = 3;
/// Max iterations for lookup to avoid infinite loops in small demos
const MAX_STEPS: usize = 8;

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

    /// Construct from a 20-byte array
    fn from_bytes(bytes: [u8; 20]) -> Self {
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

    /// Compute a key's 160-bit ID using SHA-1
    fn key_to_id(key: &[u8]) -> NodeId {
        let digest = Sha1::digest(key);
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&digest[..20]);
        NodeId::from_bytes(bytes)
    }

    /// Snapshot known peers of a node (to avoid borrow issues during iteration)
    fn snapshot_peers(&self, id: &NodeId) -> Vec<NodeId> {
        self.nodes.get(id).map(|n| n.peers.clone()).unwrap_or_default()
    }

    /// Return up to K closest nodes from `candidates` to `target` (by XOR)
    fn closest_k(&self, target: &NodeId, candidates: &[NodeId]) -> Vec<NodeId> {
        let mut list = candidates.to_vec();
        list.sort_by(|a, b| {
            let da = target.xor_distance(a);
            let db = target.xor_distance(b);
            compare_distances(&da, &db)
        });
        list.truncate(K);
        list
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

    /// Iterative find_node: start from `start`, walk the network to find K closest to `target`
    fn iterative_find_node(&mut self, start: &NodeId, target: &NodeId) -> Vec<NodeId> {
        let mut queried: Vec<NodeId> = Vec::new();
        let mut shortlist: Vec<NodeId> = self.snapshot_peers(start);
        if !shortlist.contains(start) {
            shortlist.push(*start);
        }
        shortlist = self.closest_k(target, &shortlist);

        for _step in 0..MAX_STEPS {
            // pick up to ALPHA closest not-yet-queried nodes
            let mut batch: Vec<NodeId> = Vec::new();
            for n in &shortlist {
                if !queried.contains(n) {
                    batch.push(*n);
                }
                if batch.len() == ALPHA { break; }
            }
            if batch.is_empty() { break; }

            let mut any_progress = false;
            for n in batch {
                queried.push(n);
                if let Some(neighbors) = self.find_node(start, &n, target) {
                    // merge neighbors into shortlist
                    for m in neighbors {
                        if !shortlist.contains(&m) {
                            shortlist.push(m);
                        }
                    }
                    let before = shortlist.clone();
                    shortlist = self.closest_k(target, &shortlist);
                    if shortlist != before { any_progress = true; }
                }
            }
            if !any_progress { break; }
        }
        self.closest_k(target, &shortlist)
    }

    /// Iterative find_value: like find_node but stop if a value is found
    fn iterative_find_value(&mut self, start: &NodeId, key: &[u8]) -> Option<Vec<u8>> {
        let key_id = Self::key_to_id(key);
        let mut queried: Vec<NodeId> = Vec::new();
        let mut shortlist: Vec<NodeId> = self.snapshot_peers(start);
        if !shortlist.contains(start) {
            shortlist.push(*start);
        }
        shortlist = self.closest_k(&key_id, &shortlist);

        for _step in 0..MAX_STEPS {
            let mut batch: Vec<NodeId> = Vec::new();
            for n in &shortlist {
                if !queried.contains(n) {
                    batch.push(*n);
                }
                if batch.len() == ALPHA { break; }
            }
            if batch.is_empty() { break; }

            let mut any_progress = false;
            for n in batch {
                queried.push(n);
                if let Some(result) = self.find_value(start, &n, key) {
                    if let Some(value) = result { return Some(value); }
                }
                if let Some(neighbors) = self.find_node(start, &n, &key_id) {
                    for m in neighbors {
                        if !shortlist.contains(&m) {
                            shortlist.push(m);
                        }
                    }
                    let before = shortlist.clone();
                    shortlist = self.closest_k(&key_id, &shortlist);
                    if shortlist != before { any_progress = true; }
                }
            }
            if !any_progress { break; }
        }
        None
    }

    /// Iterative store: route to K closest nodes to key_id and store there
    fn iterative_store(&mut self, start: &NodeId, key: Vec<u8>, value: Vec<u8>) {
        let key_id = Self::key_to_id(&key);
        let closest = self.iterative_find_node(start, &key_id);
        for target in closest {
            let _ = self.store(start, &target, key.clone(), value.clone());
        }
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

    // Iterative store: route to K closest to the key
    let key = b"hello".to_vec();
    let value = b"world".to_vec();
    network.iterative_store(&id1, key.clone(), value.clone());

    // Iterative find_value from id2
    let got = network.iterative_find_value(&id2, &key);
    println!(
        "Iterative find_value from node2 for 'hello': {:?}",
        got.map(|v| String::from_utf8_lossy(&v).to_string())
    );

    // Show iterative find_node for id2 starting from id0
    let closest_to_id2 = network.iterative_find_node(&id0, &id2);
    let list: Vec<String> = closest_to_id2.iter().map(|nid| Network::id_hex(nid)).collect();
    println!("Iterative closest to id2 (from id0): {:?}", list);
}
