## Step 2: In-memory network and RPC forwarding

In this step, we introduced a very simple “network” that owns multiple nodes and lets them call each other’s RPC-like methods. There is no real networking yet (no sockets); it’s all in-memory function calls, which is perfect for learning and for a small simulation.

### What we built
- **`Network`**: Holds a map from `NodeId` to `Node`.
  - `add_node()`: creates a node with a random 160-bit ID, ensures uniqueness, then registers it.
  - `ping`, `store`, `find_value`, `find_node`: forward an RPC from one node (the caller) to another node (the target) using the target’s `NodeId`.
  - `id_hex`: helper to print a `NodeId` nicely in hex.
- **`main`**: builds a `Network`, creates three nodes, prints their IDs, and demonstrates inter-node `ping`, `store`, and `find_value` through the network.

### How the network works (step-by-step)
- **Node creation (`add_node`)**
  1. Make a new `Node` with a random `NodeId`.
  2. If the `NodeId` is already used, try again (extremely unlikely).
  3. Insert the node into the `HashMap<NodeId, Node>` and return its ID.

- **Ping (`ping`)**
  1. Look up the target node by `NodeId`: `self.nodes.get(to)`.
  2. If not found, return `None` (more on `Option` below).
  3. If found, call the node’s `rpc_ping(from)` and wrap the `bool` in `Some(bool)`.

- **Store (`store`)**
  1. Look up the target node mutably: `self.nodes.get_mut(to)`.
  2. If not found, return `None`.
  3. If found, call `rpc_store(key, value)` on that node and return `Some(())`.

- **Find value (`find_value`)**
  1. Look up the target node: `self.nodes.get(to)`.
  2. If not found, return `None`.
  3. If found, call `rpc_find_value(key)`, which itself returns `Option<Vec<u8>>` (value found or not). We then wrap that in `Some(...)` because the network lookup succeeded.
     - The overall return type is `Option<Option<Vec<u8>>>`:
       - `None`: the target node wasn’t found in the network (invalid `NodeId`).
       - `Some(None)`: the target node exists, but does not have that key.
       - `Some(Some(value))`: the target node exists and returned the value.

- **Find node (`find_node`)**
  - Same pattern as above, but currently a stub inside `Node` that returns an empty list. We’ll implement routing tables later (k-buckets) so this becomes meaningful.

### What does `Some(target.rpc_ping(from))` mean?
- The method `rpc_ping` returns a `bool` (e.g., `true`).
- The network’s `ping` method returns `Option<bool>` because the target node might not exist in the network (bad `NodeId`).
- `Some(x)` wraps a value `x` inside an `Option`, signaling success.
- So `Some(target.rpc_ping(from))` means: we successfully found the target and we are returning its `bool` result inside an `Option`.

### What does the `?` do here: `self.nodes.get(to)?`?
- `get(to)` returns `Option<&Node>`.
- Using `?` on an `Option` means:
  - If it’s `Some(reference)`, unwrap and continue.
  - If it’s `None`, return `None` from the current function immediately.
- This is why our network methods return `Option<...>`: the `?` operator propagates the “not found” case naturally.

### Why return `Option` from the network?
- Looking up by `NodeId` might fail if the ID isn’t present. Representing this with `Option` makes the absence explicit.
- Separately, the underlying RPC might legitimately not find a value (e.g., key not stored), which we also model as an `Option`. That’s why some methods return nested options (e.g., `Option<Option<Vec<u8>>>`).

### How to run
```bash
cargo run
```
You should see three node IDs printed, a successful ping, and a successful lookup (since we stored the key/value on the target node).

### Where to look in the code
- `src/main.rs`
  - `struct Network` and its `impl` show the routing/forwarding logic
  - `impl Node` contains the RPC-like methods
  - `main()` demonstrates a simple scenario of inter-node calls

### What’s next (Step 3)
- Introduce a routing table (k-buckets) inside each node to track peers by XOR distance.
- Make `find_node` return the closest known nodes.
- Use this to implement iterative lookups (querying closer nodes step-by-step) and give real behavior to `find_value` beyond the local node. 