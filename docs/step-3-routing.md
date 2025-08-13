## Step 3: Simple routing (k-bucket-like peers) and `find_node`

In this step, each node maintains a small list of known peers and updates it whenever RPCs occur. We use XOR distance to sort peers and return the K closest peers from `find_node`. This is a simplified version of Kademlia’s k-buckets.

### What we built
- **Peer tracking on each node**: `peers: Vec<NodeId>`
  - LRU-like behavior: when a peer contacts a node, that peer is moved to the end of the list.
  - Maximum size `K` (we used `K = 8` for the demo): if full, we drop the least recently seen peer.
  - No self entries and no duplicates.
- **Learning peers via RPCs**: Every RPC (`ping`, `store`, `find_value`, `find_node`) calls `track_peer(from)` on the receiver, so nodes learn who contacted them.
- **XOR distance and sorting**:
  - We already had `NodeId::xor_distance(&other) -> [u8; 20]`.
  - We added `compare_distances(a, b)` to compare two big-endian 160-bit distances for sorting.
- **`find_node`**: returns up to `K` peers from the receiver that are closest to a target `NodeId` by XOR distance.

### How it works (step-by-step)
- When node A contacts node B (e.g., `ping`), B records A in its `peers` list.
- Over time, nodes learn about other nodes simply by being contacted.
- When a node receives `find_node(target)`, it sorts its `peers` by XOR distance to `target` and returns up to `K` closest peers.
- This is the foundation for iterative lookups: ask some peers for their closest nodes to the target, then ask those closer peers again, and so on, until the search converges.

### Why XOR distance?
Kademlia’s key insight is that XOR distance is a metric with useful properties for routing. Treating IDs as bitstrings, XOR captures their “difference” at the bit level, and sorting by XOR distance tends to quickly home in on the target ID.

### Code pointers
- `src/main.rs`
  - `const K: usize = 8;` bucket size for this demo
  - `struct Node { peers: Vec<NodeId>, ... }`
  - `impl Node`:
    - `track_peer(&mut self, from: &NodeId)` maintains LRU and size cap
    - `rpc_find_node(&mut self, from: &NodeId, target: &NodeId)` sorts peers by XOR distance and returns up to `K`
  - `impl Network` forwards calls mutably so that receivers can update `peers`
  - `main()` bootstrap: initial `ping`s to let nodes learn each other, then shows `find_node`

### Understanding `track_peer`
```62:75:/Users/akhileshmanda/kademlia/src/main.rs
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
```
- **`iter().position(|p| p == peer)`**: scans the vector, returns the index of the first matching element or `None`.
- **`remove(pos)`**: removes and returns the item at index `pos`, shifting later items left.
- **`push(existing)`**: appends to the end; here it moves a recently seen peer to the back (LRU effect).
- **Capacity cap**: if length exceeds `K`, `remove(0)` drops the least-recently seen peer at the front.

### How to run
```bash
cargo run
```
You should see the IDs, a successful key/value lookup, and then a printed list of IDs that node 0 considers closest to `id2`.

### What’s next (Step 4)
- Implement an iterative lookup procedure (for `find_node` and `find_value`) that queries the currently known closest peers, collects their responses, and moves closer in rounds until convergence.
- Add more realistic behaviors like timeouts and liveness checks using `ping` to evict dead peers from the routing list. 