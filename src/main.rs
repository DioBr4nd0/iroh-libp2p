use core::fmt;
use std::{ collections::HashMap, fmt::Debug, str::FromStr };
use clap:: Parser ;
use anyhow::Result;
use futures_lite::StreamExt;
use iroh::{ protocol::Router, Endpoint, NodeAddr, NodeId };
use iroh_gossip::{ net::{ Event, Gossip, GossipEvent, GossipReceiver }, proto::TopicId };
use serde::{ Deserialize, Serialize };

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long)]
    name: Option<String>,
    #[clap(short, long, default_value = "0")]
    bind_port: u16,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    Open,
    Join {
        ticket: String,
    },
}
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let (topic, nodes) = match &args.command {
        Command::Open => {
            let topic = TopicId::from_bytes(rand::random());
            println!("> opening chat room for topic {topic}");
            (topic, vec![])
        }
        Command::Join { ticket } => {
            let Ticket { topic, nodes } = Ticket::from_str(ticket)?;
            println!("> joining chat room for topic {topic}");
            (topic, nodes)
        }
    };
    // Create an endpoint.
    let endpoint = Endpoint::builder().discovery_n0().bind().await?;

    println!("> our node id: {}", endpoint.node_id());

    let gossip = Gossip::builder().spawn(endpoint.clone()).await?;

    let router = Router::builder(endpoint.clone())
        .accept(iroh_gossip::ALPN, gossip.clone())
        .spawn().await?;

    let ticket = {
        let me = endpoint.node_addr().await?;
        let nodes = vec![me];
        Ticket { topic, nodes }
    };

    println!("ticket to join us: {ticket}");
    let node_ids = nodes
        .iter()
        .map(|p| p.node_id)
        .collect();
    if nodes.is_empty() {
        println!("waiting for nodes to join us....");
    } else {
        println!("> trying to connect to {} nodes...", nodes.len());
        for node in nodes.into_iter() {
            endpoint.add_node_addr(node)?;
        }
    }
    let (sender, receiver) = gossip.subscribe_and_join(topic, node_ids).await?.split();
    println!("> connected!");

    if let Some(name) = args.name {
        let message = Message::AboutMe { from: endpoint.node_id(), name };
        sender.broadcast(message.to_vec().into()).await?;
    }

    tokio::spawn(subscribe_loop(receiver));

    let (line_tx, mut line_rx) = tokio::sync::mpsc::channel(1);
    std::thread::spawn(move || input_loop(line_tx));

    println!("> type a message and hit enter to broadcast...");

    while let Some(text) = line_rx.recv().await {
        let message = Message::Message { from: endpoint.node_id(), text: text.clone() };
        sender.broadcast(message.to_vec().into()).await?;

        println!("sent: {text}");
    }

    router.shutdown().await?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
enum Message {
    AboutMe {
        from: NodeId,
        name: String,
    },
    Message {
        from: NodeId,
        text: String,
    },
}

impl Message {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(Into::into)
    }
    pub fn to_vec(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("serde_json::to_vec is infalliable")
    }
}

async fn subscribe_loop(mut reciever: GossipReceiver) -> Result<()> {
    let mut names = HashMap::new();
    while let Some(event) = reciever.try_next().await? {
        if let Event::Gossip(GossipEvent::Received(msg)) = event {
            match Message::from_bytes(&msg.content)? {
                Message::AboutMe { from, name } => {
                    names.insert(from, name.clone());
                    println!("> {} is now known as {}", from.fmt_short(), name);
                }
                Message::Message { from, text } => {
                    let name = names.get(&from).map_or_else(|| from.fmt_short(), String::to_string);

                    println!("{}: {}", name, text);
                }
            }
        }
    }
    Ok(())
}

fn input_loop(line_tx: tokio::sync::mpsc::Sender<String>) -> Result<()> {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    loop {
        stdin.read_line(&mut buffer)?;
        line_tx.blocking_send(buffer.clone())?;
        buffer.clear();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Ticket {
    topic: TopicId,
    nodes: Vec<NodeAddr>,
}

impl Ticket {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(Into::into)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("serde_json::to_vec is infallible")
    }
}

impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut text = data_encoding::BASE32HEX_NOPAD.encode(&self.to_bytes()[..]);
        text.make_ascii_lowercase();
        write!(f, "{}", text)
    }
}

impl FromStr for Ticket {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let bytes = data_encoding::BASE32HEX_NOPAD.decode(s.to_ascii_uppercase().as_bytes())?;
        Self::from_bytes(&bytes)
    }
}
