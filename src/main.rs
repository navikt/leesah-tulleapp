use std::env;

use rdkafka::Offset::Offset;
use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::producer::FutureProducer;
use rdkafka::error::KafkaResult;
use rdkafka::message::Message;
use rdkafka::topic_partition_list::TopicPartitionList;
use rdkafka::util::{get_rdkafka_version, Timeout};
use serde::{Serialize, Deserialize};

use tracing::{warn, info};
use tracing_subscriber::prelude::*;

#[derive(Serialize, Deserialize)]
struct Payload {

}

// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.
struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        info!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        info!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        info!("Committing offsets: {:?}", result);
    }
}

// A type alias with your custom consumer can be created for convenience.
type LoggingConsumer = StreamConsumer<CustomContext>;

async fn consume_and_print(brokers: &str, group_id: &str, topic: &str, cert: &str, ca_path: &str, key: &str) {
    let context = CustomContext;

    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", group_id.clone())
        .set("bootstrap.servers", brokers.clone())
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("security.protocol", "ssl")
        .set("ssl.certificate.pem", cert.clone())
        .set("ssl.key.pem", key.clone())
        .set("ssl.ca.location", ca_path.clone())
        //.set("statistics.interval.ms", "30000")
        .set("auto.offset.reset", "smallest")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(context)
        .expect("Consumer creation failed");

    consumer.seek(topic.clone(), 0, Offset(0), Timeout::Never).unwrap();

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .set("group.id", group_id)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("security.protocol", "ssl")
        .set("ssl.certificate.pem", cert)
        .set("ssl.key.pem", key)
        .set("ssl.ca.location", ca_path)
        .create()
        .expect("Producer creation error");

    consumer
        .subscribe(&vec![topic])
        .expect("Can't subscribe to specified topics");

    loop {
        match consumer.recv().await {
            Err(e) => warn!("Kafka error: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        warn!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                info!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}

#[tokio::main]
async fn main() {
    let rust_log_env = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let filter_layer = tracing_subscriber::EnvFilter::builder().with_regex(false).parse_lossy(&rust_log_env);
    let format_layer = tracing_subscriber::fmt::layer().json().flatten_event(true);
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(format_layer)
        .init();
    let (version_n, version_s) = get_rdkafka_version();
    info!("rd_kafka_version: 0x{:08x}, {}", version_n, version_s);

    let app_name = env::var("NAIS_APP_NAME").expect("NAIS_APP_NAME must be set");
    let topics = env::var("QUIZ_TOPIC").expect("QUIZ_TOPIC must be set");
    let brokers = env::var("KAFKA_BROKERS").expect("KAFKA_BROKERS must be set");
    let group_id = env::var("QUIZRAPID_CONSUMER_GROUP").unwrap_or(format!("consumer-{app_name}-v1"));
    let cert = env::var("KAFKA_CERTIFICATE").expect("KAFKA_CERTIFICATE must be set");
    let key = env::var("KAFKA_PRIVATE_KEY").expect("KAFKA_PRIVATE_KEY must be set");
    let ca_path = env::var("KAFKA_CA_PATH").expect("KAFKA_CA_PATH must be set");


    consume_and_print(&brokers, &group_id, &topics, &cert, &ca_path, &key).await
    
}
