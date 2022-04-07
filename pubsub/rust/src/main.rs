extern crate core;

mod connection;

use crate::connection::{Connection, CHANNEL_ID_KEY, PONG_FRAME, USER_ID_KEY, PING_FRAME};
use futures_util::stream::SplitSink;
use futures_util::StreamExt;
use google_cloud_gax::cancel::CancellationToken;
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::client::Client;
use google_cloud_pubsub::publisher::Publisher;
use google_cloud_pubsub::subscription::{Subscription, SubscriptionConfig};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::convert::Infallible;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal::unix::{signal, SignalKind};
use tokio::task::JoinHandle;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

type Clients = Arc<RwLock<HashMap<String, Vec<Connection>>>>;

#[derive(Debug)]
struct InvalidParameter;

impl warp::reject::Reject for InvalidParameter {}

mod init;

#[tokio::main]
async fn main() {
    let project_id = std::env::var("PUBSUB_PROJECT").unwrap();
    init::init_trace(project_id.as_str()).await;
    let client = Client::new(&project_id, None).await.unwrap();
    let topic = client.topic("chat");
    let uuid = uuid::Uuid::new_v4().to_string();
    let subscription = client
        .create_subscription(
            &format!("ts-{}", uuid),
            topic.id().as_str(),
            SubscriptionConfig::default(),
            None,
            None,
        )
        .await
        .unwrap();
    let mut publisher = topic.new_publisher(None);

    let cons: Clients = Arc::new(RwLock::new(HashMap::<String, Vec<Connection>>::new()));
    let cons_clone = cons.clone();
    let health = warp::path("Health").map(|| format!("Server OK"));
    let operation = warp::path("Users")
        .and(warp::query())
        .and(warp::any().map(move || cons_clone.clone()))
        .map(|param: HashMap<String, String>, cons: Clients| {
            let channel_id = match param.get(CHANNEL_ID_KEY) {
                Some(v) => v.to_string(),
                None => return "invalid parameter".to_string(),
            };
            match cons.read().get(channel_id.as_str()) {
                Some(v) => format!("users={}", v.len()),
                None => "nouser".to_string(),
            }
        });

    let cons_clone = cons.clone();
    let publisher_clone = publisher.clone();
    let publisher_clone2 = publisher.clone();
    let ws = warp::path!("Connect")
        .and(warp::ws())
        .and(warp::query())
        .and(warp::any().map(move || publisher_clone.clone()))
        .and(warp::any().map(move || cons_clone.clone()))
        .and_then(handle_ws_client);

    let routes = health.or(operation).or(ws).recover(handle_rejection);

    let cancel = CancellationToken::new();
    let http_server = {
        let cancel = cancel.clone();
        let (_, server) =
            warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], 8091), async move {
                tracing::info!("Listening on ws://0.0.0.0:8091");
                cancel.cancelled().await;
                tracing::info!("Shutdown server");
            });

        tokio::spawn(server)
    };

    let subscriber = start_subscribe(cancel.clone(), subscription, cons);

    let cancel_clone = cancel.clone();
    let topic_ping = tokio::spawn(async move {
        //ping
        loop {
            tokio::select! {
                _ = cancel_clone.cancelled() => break,
                _ = tokio::time::sleep(Duration::from_secs(60)) => {
                    let result = publisher_clone2.publish(PubsubMessage {
                        data: PING_FRAME.to_vec(),
                        attributes: Default::default(),
                        message_id: "".to_string(),
                        publish_time: None,
                        ordering_key: "".to_string()
                    }).await.get(Some(cancel_clone.clone())).await;
                    if result.is_err() {
                        tracing::error!("streaming ping error {}", result.err().unwrap());
                    }
                }
            }
        }
    });
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    tokio::select! {
        _ = sigint.recv() => tracing::info!("SIGINT"),
        _ = sigterm.recv() => tracing::info!("SIGTERM"),
        _ = cancel.cancelled() => {},
    };
    if !cancel.is_cancelled() {
        cancel.cancel();
    }
    let _ = http_server.await;
    let _ = subscriber.await;
    let _ = topic_ping.abort();
    let _ = publisher.shutdown();

    tracing::info!("Shutdown complete.");
}

fn start_subscribe(
    cancel: CancellationToken,
    subscription: Subscription,
    cons: Clients,
) -> JoinHandle<()> {
    //start subscriber
    let cancel = cancel.clone();
    tokio::spawn(async move {
        let result = subscription
            .receive(
                move |message, cancel| {
                    let cons = cons.clone();
                    async move {
                        let data = message.message.data.as_slice();
                        if data == PING_FRAME {
                            tracing::info!("streaming ping message");
                            return;
                        }
                        let result = message.ack().await;
                        let channel_id = match message.message.attributes.get(CHANNEL_ID_KEY) {
                            Some(v) => v,
                            None => return,
                        };
                        let user_id = match message.message.attributes.get(USER_ID_KEY) {
                            Some(v) => v,
                            None => return,
                        };
                        on_subscribe(user_id, channel_id, cons, data);
                    }
                },
                cancel,
                None,
            )
            .await;
        if let Err(err) = result {
            tracing::error!("receive error {}", err);
        };
        let _ = subscription.delete(None, None).await;
    })
}

#[tracing::instrument(skip(cons,data))]
fn on_subscribe(
    user_id: &String,
    channel_id: &String,
    cons: Clients,
    data: &[u8],
) {
    tracing::info!("subscribe message : size={}", data.len());
    if let Some(channel_users) = cons.read().get(channel_id) {
        for user in channel_users {
            if !user.is_sender(user_id) {
                let send_result = user.send(data);
                if send_result.is_err() {
                    tracing::error!("send error {}", send_result.unwrap_err());
                }
            }
        }
    }
}

async fn handle_ws_client(
    ws: warp::ws::Ws,
    param: HashMap<String, String>,
    publisher: Publisher,
    cons: Clients,
) -> Result<impl Reply, Rejection> {
    let channel_id = match param.get(CHANNEL_ID_KEY) {
        Some(v) => v.to_string(),
        None => return Err(warp::reject::custom(InvalidParameter)),
    };
    let user_id = match param.get(USER_ID_KEY) {
        Some(v) => v.to_string(),
        None => return Err(warp::reject::custom(InvalidParameter)),
    };
    if channel_id.len() != 37 {
        return Err(warp::reject::custom(InvalidParameter));
    }

    Ok(ws.on_upgrade(|websocket| async move {
        // receiver - this server, from websocket client
        // sender - diff clients connected to this server
        let (mut sender, mut receiver) = websocket.split();
        let channel_id = channel_id;
        let user_id = user_id;
        let con = Connection::new(channel_id, user_id, sender);

        let channel_id = con.channel_id.clone();
        let user_id = con.user_id.clone();

        // add client
        {
            let mut lock = cons.write();
            if let Some(v) = lock.get_mut(con.channel_id.as_str()) {
                v.push(con);
            } else {
                lock.insert(con.channel_id.to_string(), vec![con]);
            }
        }

        while let Some(body) = receiver.next().await {
            let message = match body {
                Ok(msg) => msg,
                Err(_) => break,
            };
            let data = message.into_bytes();
            if data.as_slice() == PONG_FRAME {
                continue;
            }

            on_receive(user_id.clone(), channel_id.clone(), &publisher, data).await
        }

        tracing::info!("client disconnected");

        // remove client
        let mut lock = cons.write();
        if let Some(v) = lock.get_mut(channel_id.as_str()) {
            v.retain(|x| !x.is_sender(user_id.as_str()));
            tracing::info!("client count in channel {} is {}", channel_id, v.len());
        }
    }))
}

#[tracing::instrument(skip(publisher,data))]
async fn on_receive(
    user_id: String,
    channel_id: String,
    publisher: &Publisher,
    data: Vec<u8>
) {
    tracing::info!("receive message : size={}", data.len());
    let mut attributes = HashMap::<String, String>::new();
    attributes.insert(CHANNEL_ID_KEY.to_string(), channel_id);
    attributes.insert(USER_ID_KEY.to_string(), user_id);

    let mut msg = PubsubMessage {
        data,
        attributes,
        message_id: "".to_string(),
        publish_time: None,
        ordering_key: "".to_string(),
    };
    let result = publisher.publish(msg).await.get(None).await;
    if result.is_err() {
        tracing::error!("error publish message: {}", result.unwrap_err());
    }
}


async fn handle_rejection(
    err: warp::reject::Rejection,
) -> std::result::Result<impl warp::reply::Reply, Infallible> {
    let code;
    let message;
    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not found";
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = "Invalid Body";
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Method not allowed";
    } else if let Some(_) = err.find::<InvalidParameter>() {
        code = StatusCode::BAD_REQUEST;
        message = "invalid parameter";
    } else {
        // We should have expected this... Just log and say its a 500
        tracing::error!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal server error";
    }

    let json = warp::reply::html(message);

    Ok(warp::reply::with_status(json, code))
}
