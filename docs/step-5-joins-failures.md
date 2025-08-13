## Step 5: Joins and failures (liveness and eviction)

In this step, we simulate node joins and failures and make our iterative procedures resilient by evicting dead peers from routing lists.

### What we built
- **Liveness flag**: each `Node` has `alive: bool`. This simulates whether the node can respond.
- **Join helper**: `add_and_join(bootstrap)` adds a new node, pings the bootstrap so both learn about each other, then runs an iterative `find_node` towards its own ID to discover neighbors.
- **Failure injection**: `kill_node(id)` sets `alive = false` for a node.
- **Eviction**: when a peer is unreachable (ping fails), we remove it from all nodes’ peer lists with `evict_peer_from_all(peer)`.
- **Lookup robustness**: iterative procedures (`iterative_find_node`, `iterative_find_value`, `iterative_store`) check liveness via `ping` and skip/evict dead peers to keep routing clean.

### How liveness is checked
- All RPC forwarders (`ping`, `store`, `find_value`, `find_node`) first confirm the target is `alive`. If not, they return `None`.
- During iterative procedures, we proactively `ping` a candidate before querying it. If the ping isn’t `Some(true)`, we evict the peer.

### Why eviction matters
Dead peers degrade routing quality and slow convergence. Removing them when detected keeps the peer lists fresh and focused on reachable nodes, so iterative lookups complete faster and more reliably.

### Flow examples
- **Joining**
  1. Create node N.
  2. Ping bootstrap B so B learns about N (and N tracks B).
  3. Run `iterative_find_node(N, N.id)` to gather N’s initial neighborhood.

- **Failure**
  1. Mark node X dead (`kill_node(X)`).
  2. Future lookups that encounter X will ping, see it’s unreachable, and evict X from all peer lists.

### Code pointers
- `src/main.rs`
  - `struct Node { alive: bool, ... }`
  - `Network::add_and_join`, `kill_node`, `evict_peer_from_all`
  - RPC forwarders check `alive`
  - Iterative functions ping before asking for neighbors/values and evict on failure
  - `main()` demonstrates: iterative store, join via bootstrap, kill a node, then lookups that proceed despite the failure

### How to run
```bash
cargo run
```
You’ll see:
- Node IDs, a store operation,
- A new node joining via bootstrap,
- A simulated failure,
- Iterative lookups that still succeed while skipping/evicting dead peers.

### Next ideas
- Periodic refresh of buckets (background tasks) and regular liveness checks.
- Real networking (sockets), timeouts, and retries.
- More realistic bucket structure with per-prefix k-buckets rather than a single Vec. 