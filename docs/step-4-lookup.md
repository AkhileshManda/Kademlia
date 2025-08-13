## Step 4: Iterative lookup and store

Now that nodes can learn peers and return the K closest nodes by XOR distance, we implement iterative procedures that “walk” the network to find nodes and values. We also map keys to 160-bit IDs using SHA-1 so the distance metric applies to keys as well.

### What we built
- **Key IDs via SHA-1**: `key_to_id(key: &[u8]) -> NodeId` computes a 160-bit ID for a key using SHA-1. This lets us treat keys and nodes in the same ID space.
- **Closest-K helper**: `closest_k(target, candidates)` sorts by XOR distance and returns up to `K` items.
- **Peer snapshots**: `snapshot_peers(id)` returns a copy of a node’s peer list to avoid borrowing issues during iteration.
- **Iterative find_node**: starting from a node, repeatedly query up to `ALPHA` closest unqueried nodes for neighbors, merge, re-sort, and continue until no progress or a step limit. Returns K closest nodes to the target.
- **Iterative find_value**: same as find_node, but if any queried node returns a value, stop and return it.
- **Iterative store**: route a key/value to the K nodes closest to the key’s ID (by running iterative find_node first), then store on those nodes.

### Parameters
- `K = 8`: bucket/answer size in this demo (smaller than typical Kademlia’s 20).
- `ALPHA = 3`: query concurrency factor in Kademlia; we still call sequentially, but batch selection follows alpha.
- `MAX_STEPS = 8`: hard stop to prevent infinite loops in tiny test networks.

### High-level flow
- Start with a shortlist: the caller’s known peers plus itself, sorted by XOR distance to the target ID.
- Repeatedly pick up to `ALPHA` closest nodes that have not been queried yet.
- For each selected node:
  - `find_value`: ask for the value; if found, return it immediately.
  - Ask for neighbors `find_node(target)`; merge neighbors into the shortlist.
- Re-sort shortlist, keep only K closest, and repeat until no improvement.

### Why SHA-1 for keys?
We need keys to live in the same metric space as node IDs to compare distances. SHA-1 gives a 160-bit digest, matching our `NodeId` size. For learning purposes, it’s perfect here. (In production, stronger hashes may be preferred.)

### Code pointers
- `src/main.rs`
  - Constants: `K`, `ALPHA`, `MAX_STEPS`
  - `Network::key_to_id` (SHA-1)
  - `Network::closest_k`, `snapshot_peers`
  - `Network::iterative_find_node`, `iterative_find_value`, `iterative_store`
  - `main()` shows example: iterative store and lookups

### How to run
```bash
cargo run
```
You should see iterative lookups returning values and lists of closest nodes.

### What’s next (Step 5)
- Node lifecycle: join procedures, and simple failure handling (removing dead peers on failed pings).
- Optional: timeouts and retries; background refresh of buckets; more realistic networking. 