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

fn main() {
    // Create a few nodes and print their IDs
    let mut nodes: Vec<Node> = (0..3).map(|_| Node::new()).collect();

    for (i, node) in nodes.iter().enumerate() {
        // Print ID as hex for readability
        let hex_id: String = node.id.0.iter().map(|b| format!("{b:02x}")).collect();
        println!("Node {i}: {hex_id}");
    }

    // Minimal demo of RPC-like methods locally (we'll add inter-node comms next)
    let key = b"hello".to_vec();
    let value = b"world".to_vec();

    // Store on node 0
    nodes[0].rpc_store(key.clone(), value.clone());

    // Find value on node 0
    let got = nodes[0].rpc_find_value(&key);
    println!("Lookup on node 0: {:?}", got.map(|v| String::from_utf8_lossy(&v).to_string()));
}
