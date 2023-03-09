use std::time::Duration;

use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use warp::ws::Message;
use warp::ws::WebSocket;

pub const CHANNEL_ID_KEY: &str = "channelId";
pub const USER_ID_KEY: &str = "userId";
pub const PING_FRAME: &[u8] = &[0x09];
pub const PONG_FRAME: &[u8] = &[0x10];

pub struct Connection {
    pub channel_id: String,
    pub user_id: String,
    queue: UnboundedSender<Vec<u8>>,
    task: JoinHandle<()>,
}

impl Connection {
    pub fn new(channel_id: String, user_id: String, mut sender: SplitSink<WebSocket, warp::ws::Message>) -> Self {
        tracing::info!("client connected channel={} user={}", channel_id, user_id);
        let (tx, mut rx) = mpsc::unbounded_channel();

        let channel_id_clone = channel_id.clone();
        let user_id_clone = user_id.clone();
        let task = tokio::spawn(async move {
            loop {
                if !send(user_id_clone.as_str(), channel_id_clone.as_str(), &mut sender, &mut rx).await {
                    break;
                }
            }
        });

        Self {
            channel_id,
            user_id,
            task,
            queue: tx,
        }
    }

    pub fn is_sender(&self, user_id: &str) -> bool {
        self.user_id.as_str() == user_id
    }

    pub fn send(&self, data: &[u8]) -> Result<(), SendError<Vec<u8>>> {
        self.queue.send(data.to_vec())
    }
}

#[tracing::instrument(skip(sender, rx))]
async fn send(
    user_id: &str,
    channel_id: &str,
    sender: &mut SplitSink<WebSocket, warp::ws::Message>,
    rx: &mut UnboundedReceiver<Vec<u8>>,
) -> bool {
    match timeout(Duration::from_secs(10), rx.recv()).await {
        Ok(maybe_message) => {
            match maybe_message {
                Some(msg) => {
                    tracing::info!("send to remote target : size={}", msg.len());
                    let text = match String::from_utf8(msg) {
                        Ok(text) => text,
                        Err(_e) => return true,
                    };
                    let result = sender.send(Message::text(text)).await;
                    if let Err(e) = result {
                        tracing::error!("broadcast error {}", e);
                    }
                    true
                }
                None => {
                    //closed
                    false
                }
            }
        }
        //ping
        Err(_err) => {
            let result = sender.send(Message::binary(PING_FRAME)).await;
            if let Err(e) = result {
                tracing::error!("ping error {}", e)
            }
            true
        }
    }
}
