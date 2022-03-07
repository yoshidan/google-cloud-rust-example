use google_cloud_pubsub::client::Client;

use google_cloud_gax::cancel::CancellationToken;
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::publisher::Publisher;
use google_cloud_pubsub::subscription::SubscriptionConfig;
use google_cloud_pubsub::topic::Topic;
use std::collections::HashMap;
use std::convert::Infallible;

use tokio::signal::unix::{signal, SignalKind};
use uuid::Uuid;
use warp::http::StatusCode;
use warp::reply::with_status;
use warp::{reject, Filter};
use warp::{Rejection, Reply};

async fn publish_handler(ordering_key: String, message: String, topic: Publisher) -> Result<impl Reply, Rejection> {
    let awaiter = topic
        .publish(PubsubMessage {
            data: message.as_bytes().to_vec(),
            attributes: Default::default(),
            message_id: "".to_string(),
            publish_time: None,
            ordering_key,
        })
        .await;
    let id = awaiter.get(None).await;
    match id {
        Ok(id) => Ok(warp::reply::with_status(warp::reply::html(id), warp::http::StatusCode::OK)),
        Err(e) => Ok(with_status(warp::reply::html(e.to_string()), StatusCode::INTERNAL_SERVER_ERROR)),
    }
}

fn with_publisher(publisher: Publisher) -> impl Filter<Extract = (Publisher,), Error = Infallible> + Clone {
    warp::any().map(move || publisher.clone())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let project_id = std::env::var("PUBSUB_PROJECT").unwrap();
    env_logger::init();

    let client = Client::new(&project_id, None).await.unwrap();
    let topic = client.topic("test-topic1");
    let mut publisher = topic.new_publisher(None);

    let uuid = uuid::Uuid::new_v4().to_string();
    log::info!("Start server {}.", uuid);
    let mut config = SubscriptionConfig::default();
    config.enable_message_ordering = true;
    let subscription = client
        .create_subscription(&format!("s-{}", uuid.clone()),
            topic.id().as_str(),
            config,
            None,
            None,
        )
        .await
        .unwrap();

    //define routes
    let handler = warp::path!("Publish")
        .and(warp::body::form())
        .and(with_publisher(publisher.clone()))
        .and_then(move |param: HashMap<String, String>, topic| {
            publish_handler("orderkey".to_string(), param.get("message").unwrap().to_string(), topic)
        });
    let routes = warp::post().and(handler);

    // launch http server
    let cancel = CancellationToken::new();
    let http_server = {
        let cancel = cancel.clone();
        let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], 3031), async move {
            log::info!("Listening on http://0.0.0.0:3031");
            cancel.cancelled().await;
            log::info!("Shutdown server");
        });

        tokio::spawn(server)
    };

    //start subscriber
    let subscriber = {
        let cancel = cancel.clone();
        tokio::spawn(async move {
            subscription
                .receive(
                    |mut message, cancel| async move {
                        log::info!("received {}", String::from_utf8_lossy(message.message.data.as_slice()).to_string());
                        let result = message.ack().await;
                        log::info!("ack {:?}", result)
                    },
                    cancel,
                    None,
                )
                .await;
            let _ = subscription.delete(None, None).await;
        })
    };

    // wait for signal
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    tokio::select! {
        _ = sigint.recv() => println!("SIGINT"),
        _ = sigterm.recv() => println!("SIGTERM"),
    };
    cancel.cancel();
    let _ = subscriber.await;
    let _ = http_server.await;
    publisher.shutdown().await;
    log::info!("All the spanner sessions are deleted.");
}
