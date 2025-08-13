## Step 1: Create basic nodes with IDs

This step sets up the foundation for a Kademlia DHT simulation by creating nodes that each have a unique 160-bit identifier and a tiny local key-value store. We also stubbed the core RPC calls we will flesh out later.

### What we built
- **`NodeId`**: A 160-bit identifier (20 bytes), similar to Kademlia’s ID space (often based on SHA-1). We generate random IDs using the `rand` crate.
- **`Node`**: Holds a `NodeId` and a simple in-memory `storage` (`HashMap<Vec<u8>, Vec<u8>>`). This simulates a node that can store key/value pairs.
- **RPC-like methods** (local stubs for now):
  - `rpc_ping(&self, _from: &NodeId) -> bool`: always returns true; will be used to check liveness.
  - `rpc_store(&mut self, key: Vec<u8>, value: Vec<u8>)`: store a value locally.
  - `rpc_find_value(&self, key: &[u8]) -> Option<Vec<u8>>`: lookup a value locally.
  - `rpc_find_node(&self, _target: &NodeId) -> Vec<NodeId>`: placeholder for returning closest known nodes; will be implemented when we track peers.

In `main`, we create a few nodes, print their hex IDs, store a value in one node, and look it up.

### Why 160-bit IDs?
Kademlia uses XOR distance over node IDs. We mimic that space with 160-bit IDs. We also added a helper `xor_distance` method on `NodeId` to compute XOR distances, which we’ll use when we implement routing and lookups.

### Rust concepts (beginner-friendly)
- **Structs**: Custom data types. We created `NodeId` and `Node`.
- **`impl` blocks**: Where you add methods to a struct (constructors and functions).
- **`derive` attributes**: Auto-implement useful traits (like `Debug`, `Copy`, `Eq`, `Hash`) to print and compare values without writing boilerplate.
- **Crates/dependencies**: In `Cargo.toml`, we added `rand = "0.8"` to get random bytes for IDs.
- **Vectors and slices**: `Vec<u8>` is a growable byte array; `&[u8]` is a borrowed read-only view. We store owned `Vec<u8>` in the node’s map and accept `&[u8]` for lookups.

### How to run
```bash
cargo run
```
You should see three node IDs printed in hex and a lookup result like `Some("world")`:
```text
Node 0: 1f2a...
Node 1: a7bc...
Node 2: 93de...
Lookup on node 0: Some("world")
```
(Your IDs will differ because they’re random.)

### Where to look in the code
- `Cargo.toml`: added the `rand` dependency
- `src/main.rs`: defines `NodeId`, `Node`, and the RPC-like method stubs; runs a tiny demo in `main()`

### What’s next (Step 2)
We’ll introduce a simple in-memory “network” so that nodes can call each other’s RPC methods (ping, store, find_value, find_node) rather than calling themselves directly. This will set us up to add realistic behavior and routing in later steps. 