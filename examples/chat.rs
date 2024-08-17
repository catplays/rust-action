use anyhow::Result;
use core::fmt;
use dashmap::DashMap;
use futures::{stream::SplitStream, SinkExt as _, StreamExt};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[derive(Debug, Default)]
struct ChatState {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

#[derive(Debug)]
enum Message {
    UserJoined(String),
    UserLeft(String),
    Chat { sender: String, content: String },
}

#[derive(Debug)]
struct Peer {
    username: String,
    stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}

const MAX_MESSAGES: usize = 128;

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let addr = "0.0.0.0:8080";

    let listener = TcpListener::bind(addr).await?;
    info!("Start char server on port:{}", addr);
    let state = Arc::new(ChatState::default());
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Accepted connection from: {}", addr);
        let cloned_state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = handle_client(cloned_state, addr, stream).await {
                warn!("Failed to handle client {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(state: Arc<ChatState>, addr: SocketAddr, stream: TcpStream) -> Result<()> {
    let mut stream = Framed::new(stream, LinesCodec::new());

    // 按帧处理数据，LinesCodec作用：在发送消息时自动追加\n
    stream.send("Enter your username:").await?;
    // stream.next() returns next message
    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };
    info!("username:{}", username);
    let mut peer = state.add(addr, username, stream).await;
    // broascast message to all client
    let message = Arc::new(Message::user_joined(&peer.username));
    state.broadcast(addr, message).await;
    // loop to receive message
    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("Failed to read line from {}: {}", addr, e);
                break;
            }
        };
        // build message to broadcast
        let message = Arc::new(Message::chat(&peer.username, line));
        state.broadcast(addr, message).await;
    }
    // when while loop exit,peer has left the chat
    // remove peer
    state.peers.remove(&addr);
    // notify others that a user has left
    let message = Arc::new(Message::user_left(&peer.username));
    info!("{}", message);

    state.broadcast(addr, message).await;

    Ok(())
}

impl ChatState {
    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() == &addr {
                continue;
            }
            if let Err(err) = peer.value().send(message.clone()).await {
                warn!("Failed to send message to :{} err:{}", peer.key(), err);
            }
        }
    }

    async fn add(
        &self,
        addr: SocketAddr,
        username: String,
        stream: Framed<TcpStream, LinesCodec>,
    ) -> Peer {
        // create a channel， sender bind to addr;
        let (tx, mut rx) = mpsc::channel(MAX_MESSAGES);
        self.peers.insert(addr, tx);

        let (mut stream_sender, stream_receiver) = stream.split();
        // receive message from others, and send them to client
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("Failed to send message to {}: {}", addr, e);
                    break;
                }
            }
        });
        Peer {
            username,
            stream: stream_receiver, // recv message from client
        }
    }
}

impl Message {
    fn user_joined(username: &str) -> Self {
        let content = format!("[{}] has joined the chat", username);
        Self::UserJoined(content)
    }

    fn user_left(username: &str) -> Self {
        let content = format!("[{}] has left the chat", username);
        Self::UserLeft(content)
    }

    fn chat(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Chat {
            sender: sender.into(),
            content: content.into(),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserJoined(content) => write!(f, "({})", content),
            Self::UserLeft(content) => write!(f, "({} :( )", content),
            Self::Chat { sender, content } => write!(f, "[{}]: {}", sender, content),
        }
    }
}
