use rand::Rng;
use std::collections::HashMap;

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

/// A basic node in the DHT
#[derive(Debug)]
struct Node {
    id: NodeId,
    storage: HashMap<Vec<u8>, Vec<u8>>, // very simple key-value store
}

impl Node {
    /// Create a new node with a random ID
    fn new() -> Self {
        Self {
            id: NodeId::random(),
            storage: HashMap::new(),
        }
    }

    /// RPC: ping - used to check liveness
    fn rpc_ping(&self, _from: &NodeId) -> bool {
        true
    }

    /// RPC: store - store a key/value locally
    fn rpc_store(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.storage.insert(key, value);
    }

    /// RPC: find_value - try to get a value for a key
    fn rpc_find_value(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage.get(key).cloned()
    }

    /// RPC: find_node - return known nodes closest to the target id
    /// For now, we don't track peers yet, so return empty. We'll fill this in later.
    fn rpc_find_node(&self, _target: &NodeId) -> Vec<NodeId> {
        Vec::new()
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
    fn ping(&self, from: &NodeId, to: &NodeId) -> Option<bool> {
        let target = self.nodes.get(to)?;
        Some(target.rpc_ping(from))
    }

    /// RPC forwarding: store a key/value on a target node
    fn store(&mut self, from: &NodeId, to: &NodeId, key: Vec<u8>, value: Vec<u8>) -> Option<()> {
        let target = self.nodes.get_mut(to)?;
        // from is unused now, but will be useful for routing/permissions later
        let _ = from;
        target.rpc_store(key, value);
        Some(())
    }

    /// RPC forwarding: find_value on a target node
    fn find_value(&self, from: &NodeId, to: &NodeId, key: &[u8]) -> Option<Option<Vec<u8>>> {
        let target = self.nodes.get(to)?;
        let _ = from;
        Some(target.rpc_find_value(key))
    }

    /// RPC forwarding: find_node on a target node
    fn find_node(&self, from: &NodeId, to: &NodeId, target_id: &NodeId) -> Option<Vec<NodeId>> {
        let target = self.nodes.get(to)?;
        let _ = from;
        Some(target.rpc_find_node(target_id))
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

    // Inter-node RPC calls through the network
    let alive = network.ping(&id1, &id0).unwrap_or(false);
    println!("Ping from 1 -> 0: {}", alive);

    let key = b"hello".to_vec();
    let value = b"world".to_vec();

    // Node 1 asks Node 0 to store a value
    network.store(&id1, &id0, key.clone(), value.clone());

    // Node 2 asks Node 0 to look up the value
    let got = network.find_value(&id2, &id0, &key).unwrap_or(None);
    println!("Lookup on node 0 (asked by 2): {:?}", got.map(|v| String::from_utf8_lossy(&v).to_string()));
}
