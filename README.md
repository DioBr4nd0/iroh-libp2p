# Iroh serverless P2P Communication 

A demonstration of peer-to-peer networking using Rust and IROH, showcasing how nodes can communicate without a central server.

## What is P2P?

In traditional client-server architectures, all communication goes through a central server. P2P networks instead allow direct communication between participants (peers):

```
Client-Server:          P2P Network:
    Server                 Peer ←→ Peer
    ↙ ↓ ↘               ↙   ↓   ↘
Client Client Client   Peer ←→ Peer ←→ Peer
```

## How it Works

This example implements:

1. **Node Discovery**: Peers find each other using shareable tickets containing network addresses
2. **Gossip Protocol**: Messages spread through the network like rumors:
   - When a peer receives new information, it shares with its neighbors
   - Those neighbors share with their neighbors
   - Process continues until all peers have the information

3. **Distributed Communication**: Any peer can:
   - Create a new topic
   - Join existing topics
   - Broadcast messages to all connected peers
   - Maintain peer-to-peer connections without central coordination

## Technical Implementation

Built using:
- **IROH**: Modern P2P networking stack
- **Tokio**: Asynchronous runtime
- **Gossip Protocol**: For reliable message propagation
- **Serde**: Data serialization
- **Clap**: Command line interface

## Try It Out

Create a new peer node:
```bash
cargo run -- open
```

Join an existing peer network:
```bash
cargo run -- join <ticket>
```

## Use Cases

This pattern can be applied to build:
- Distributed databases
- File sharing networks
- Decentralized social networks
- Blockchain networks
- Edge computing systems

## Dependencies
```toml
[dependencies]
iroh = "0.1"
iroh-gossip = "0.1"
tokio = { version = "1.0", features = ["full"] }
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
```

## References
[Iroh docs](https://www.iroh.computer/blog/comparing-iroh-and-libp2p)