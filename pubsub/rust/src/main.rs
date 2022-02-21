use google_cloud_pubsub::client::Client;

use std::collections::HashMap;
use std::convert::Infallible;
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::subscription::SubscriptionConfig;
use google_cloud_pubsub::topic::Topic;

use tokio::signal::unix::{signal, SignalKind};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use warp::{Filter, reject};
use warp::{Rejection, Reply};
use warp::http::StatusCode;
use warp::reply::with_status;

async fn publish_handler(ordering_key: String, user_id: String, topic: Topic) -> Result<impl Reply, Rejection> {
    let awaiter = topic.publish(PubsubMessage {
        data: user_id.as_bytes().to_vec(),
        attributes: Default::default(),
        message_id: "".to_string(),
        publish_time: None,
        ordering_key,
    }).await;
    let id = awaiter.get(CancellationToken::new()).await;
    match id {
        Ok(id) => Ok(warp::reply::with_status(warp::reply::html(id), warp::http::StatusCode::OK)),
        Err(e) => Ok(with_status(warp::reply::html(e.to_string()), StatusCode::INTERNAL_SERVER_ERROR))
    }
}

fn with_topic(topic: Topic) -> impl Filter<Extract = (Topic, ), Error = Infallible> + Clone {
    warp::any().map(move || topic.clone())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let project_id = std::env::var("PUBSUB_PROJECT").unwrap();
    env_logger::init();

    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::terminate()).unwrap();

    let client = Client::new(&project_id, None).await.unwrap();
    let ctx = CancellationToken::new();
    let topic = client.topic("test-topic1", None);

    let uuid = uuid::Uuid::new_v4().to_string();
    log::info!("Start server {}.", uuid);
    let subscription = client.create_subscription(ctx.clone(), &format!("s-{}", uuid.clone()), topic.string(), SubscriptionConfig::default(), None).await.unwrap();
    let mut config = SubscriptionConfig::default();
    config.enable_message_ordering = true;
    let ordered_subscription = client.create_subscription(ctx.clone(), &format!("so-{}",uuid), topic.string(), config, None).await.unwrap();

    //define routes
    let p1 = warp::path!("PublishOrdered")
        .and(warp::body::form())
        .and(with_topic(topic.clone()))
        .and_then(move |param: HashMap<String, String>, topic | {
            publish_handler("order".to_string(), param.get("user_id").unwrap().to_string(), topic)
        });
    let p2 = warp::path!("Publish")
        .and(warp::body::form())
        .and(with_topic( topic.clone()))
        .and_then(move |param: HashMap<String, String>, topic | {
            publish_handler("".to_string(), param.get("user_id").unwrap().to_string(), topic)
        });
    let routes = warp::post().and(p1.or(p2));

    // launch server
    let ctx0 = ctx.clone();
    let (_, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], 3031), async move {
            log::info!("Listening on http://0.0.0.0:3031");
            ctx0.cancelled().await;
            log::info!("Shutdown server");
        });

    let server_handler = tokio::spawn(server);

    let ctx1 = ctx.clone();
    let s1_handler = tokio::spawn(async move {
        subscription.receive(ctx1.clone(), |mut message, ctx| async move {
            println!("random {:?}", message.message.data);
            message.ack();
        }, None).await;
        subscription.delete(ctx1,None).await;
    });
    let ctx2 = ctx.clone();
    let s2_handler = tokio::spawn(async move {
        ordered_subscription.receive(ctx2.clone(), |mut message, ctx| async move {
            println!("ordered {:?}", message.message.data);
            message.ack();
        }, None).await;
        ordered_subscription.delete(ctx2,None).await;
    });

    // wait for signal
    tokio::select! {
        _ = sigint.recv() => println!("SIGINT"),
        _ = sigterm.recv() => println!("SIGTERM"),
    };
    ctx.cancel();
    s1_handler.await;
    s2_handler.await;
    server_handler.await;
    topic.shutdown();
    log::info!("All the spanner sessions are deleted.");
}
