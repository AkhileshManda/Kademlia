# Kademlia Implementation

This project is a Rust implementation of the Kademlia distributed hash table (DHT) protocol. It demonstrates key concepts of the Kademlia protocol, including node discovery, iterative lookups, and value storage.

## Features

- **Node Discovery**: Nodes can discover peers and maintain a list of known nodes.
- **Iterative Lookup**: Implements iterative procedures to find nodes and values by walking the network.
- **Value Storage**: Allows storing and retrieving key-value pairs across the network.
- **SHA-1 Key IDs**: Keys are mapped to 160-bit IDs using SHA-1, allowing them to be treated in the same ID space as nodes.

## How It Works

1. **Key IDs via SHA-1**: The function `key_to_id(key: &[u8]) -> NodeId` computes a 160-bit ID for a key using SHA-1.
2. **Closest-K Helper**: The function `closest_k(target, candidates)` sorts nodes by XOR distance and returns up to `K` closest nodes.
3. **Peer Snapshots**: The function `snapshot_peers(id)` returns a copy of a node’s peer list to avoid borrowing issues during iteration.
4. **Iterative Find Node**: Starting from a node, it queries up to `ALPHA` closest unqueried nodes for neighbors, merges, re-sorts, and continues until no progress or a step limit is reached.
5. **Iterative Find Value**: Similar to find_node, but stops and returns if a value is found.
6. **Iterative Store**: Routes a key/value to the K nodes closest to the key’s ID and stores it there.

## Parameters

- `K = 8`: Bucket/answer size in this demo.
- `ALPHA = 3`: Query concurrency factor.
- `MAX_STEPS = 8`: Maximum steps to prevent infinite loops.

## Running the Project

To run the project, use the following command:

```bash
cargo run
```

You should see iterative lookups returning values and lists of closest nodes.

## Documentation

The project documentation is divided into several steps:

- **Step 1**: Node creation and basic operations.
- **Step 2**: Network setup and peer discovery.
- **Step 3**: Routing and message handling.
- **Step 4**: Iterative lookup and store (current step).
- **Step 5**: Handling node joins and failures.

## License

This project is licensed under the MIT License. 