use std::{collections::HashMap, fmt, str::FromStr};

use anyhow::Result;
use chrono::Local;
use clap::Parser;
use futures_lite::StreamExt;
use iroh::{protocol::Router, Endpoint, EndpointAddr, EndpointId, endpoint::presets};
use iroh_gossip::{
    api::{Event, GossipReceiver},
    net::Gossip,
    proto::TopicId,
};
use rand::RngExt;
use serde::{Deserialize, Serialize};

fn timestamp() -> String {
    Local::now().format("[%H:%M:%S]").to_string()
}

const ADJECTIVES: &[&str] = &[
    "anonymous", "brave", "calm", "clever", "eager",
    "fierce", "gentle", "happy", "jolly", "keen",
    "lively", "mighty", "noble", "proud", "quick",
    "silent", "swift", "witty", "bold", "wild",
];

const NOUNS: &[&str] = &[
    "fox", "wolf", "hawk", "bear", "deer",
    "otter", "lynx", "crow", "stag", "owl",
    "pike", "wren", "hare", "seal", "raven",
    "falcon", "badger", "coyote", "robin", "viper",
];

fn generate_name() -> String {
    let mut rng = rand::rng();
    let adj = ADJECTIVES[rng.random_range(0..ADJECTIVES.len())];
    let noun = NOUNS[rng.random_range(0..NOUNS.len())];
    let num: u16 = rng.random_range(0..100);
    format!("{adj}-{noun}-{num}")
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, default_value = "0")]
    bind_port: u16,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    Open {
        #[clap(short, long)]
        name: Option<String>,
    },
    Join { ticket: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let (topic, endpoints, local_name) = match &args.command {
        Command::Open { name } => {
            let topic = TopicId::from_bytes(rand::random());
            println!("{} > opening chat room for topic {topic}", timestamp());
            let local_name = name.clone().unwrap_or_else(generate_name);
            (topic, vec![], local_name)
        }
        Command::Join { ticket } => {
            let Ticket { topic, endpoints } = match Ticket::from_str(ticket) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("> error: invalid ticket format — {e}");
                    eprintln!("> expected a base32-encoded ticket string");
                    std::process::exit(1);
                }
            };
            println!("{} > joining chat room for topic {topic}", timestamp());
            let local_name = generate_name();
            (topic, endpoints, local_name)
        }
    };

    let endpoint = Endpoint::bind(presets::N0).await?;

    println!("{} > our endpoint id: {}", timestamp(), endpoint.id());
    let gossip = Gossip::builder().spawn(endpoint.clone());

    let router = Router::builder(endpoint.clone())
        .accept(iroh_gossip::ALPN, gossip.clone())
        .spawn();

    let ticket = {
        let me = endpoint.addr();
        let endpoints = vec![me];
        Ticket { topic, endpoints }
    };
    println!("{} > ticket to join us: {ticket}", timestamp());

    let endpoint_ids = endpoints.iter().map(|p| p.id).collect();
    if endpoints.is_empty() {
        println!("{} > waiting for endpoints to join us...", timestamp());
    } else {
        println!("{} > trying to connect to {} endpoints...", timestamp(), endpoints.len());
    };
    let (sender, receiver) = gossip.subscribe_and_join(topic, endpoint_ids).await?.split();
    println!("{} > connected!", timestamp());

    let message = Message::new(MessageBody::AboutMe {
        from: endpoint.id(),
        name: local_name.clone(),
    });
    sender.broadcast(message.to_vec().into()).await?;

    tokio::spawn(subscribe_loop(receiver));

    let (line_tx, mut line_rx) = tokio::sync::mpsc::channel(1);
    std::thread::spawn(move || input_loop(line_tx));

    println!("{} > type a message and hit enter to broadcast...", timestamp());
    loop {
        tokio::select! {
            text = line_rx.recv() => {
                match text {
                    Some(text) => {
                        let message = Message::new(MessageBody::Message {
                            from: endpoint.id(),
                            text: text.clone(),
                        });
                        sender.broadcast(message.to_vec().into()).await?;
                        println!("{} > {local_name}: {text}", timestamp());
                    }
                    None => break,
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!();
                println!("> shutting down...");
                drop(sender);
                break;
            }
        }
    }

    router.shutdown().await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    body: MessageBody,
    nonce: [u8; 16],
}

#[derive(Debug, Serialize, Deserialize)]
enum MessageBody {
    AboutMe { from: EndpointId, name: String },
    Message { from: EndpointId, text: String },
}

impl Message {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(Into::into)
    }

    pub fn new(body: MessageBody) -> Self {
        Self {
            body,
            nonce: rand::random(),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("serde_json::to_vec is infallible")
    }
}

async fn subscribe_loop(mut receiver: GossipReceiver) -> Result<()> {
    let mut names = HashMap::new();
    while let Some(event) = receiver.try_next().await? {
        if let Event::Received(msg) = event {
            match Message::from_bytes(&msg.content)?.body {
                MessageBody::AboutMe { from, name } => {
                    names.insert(from, name.clone());
                    println!("{} > {} is now known as {}", timestamp(), from.fmt_short(), name);
                }
                MessageBody::Message { from, text } => {
                    let name = names
                        .get(&from)
                        .map_or_else(|| from.fmt_short().to_string(), String::to_string);
                    println!("{} {name}: {text}", timestamp());
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
        let line = buffer.trim_end().to_string();
        if line.is_empty() {
            buffer.clear();
            continue;
        }
        line_tx.blocking_send(line)?;
        buffer.clear();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Ticket {
    topic: TopicId,
    endpoints: Vec<EndpointAddr>,
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
        let mut text = data_encoding::BASE32_NOPAD.encode(&self.to_bytes()[..]);
        text.make_ascii_lowercase();
        write!(f, "{text}")
    }
}

impl FromStr for Ticket {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = data_encoding::BASE32_NOPAD.decode(s.to_ascii_uppercase().as_bytes())?;
        Self::from_bytes(&bytes)
    }
}
