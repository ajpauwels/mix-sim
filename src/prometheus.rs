use prometheus_client::{
    encoding::{text::encode, EncodeLabelSet, EncodeLabelValue},
    metrics::{counter::Counter, family::Family},
    registry::Registry,
};
use tiny_http::Response;
use tokio::task::JoinHandle;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct MessageLabels {
    pub from: String,
    pub to: String,
    pub status: MessageStatus,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue)]
pub enum MessageStatus {
    Sent,
    Received,
}

pub struct MetricFamilies {
    pub messages: Family<MessageLabels, Counter>,
    // pub messages_sent: Family<MessageLabels, Counter>,
    // pub messages_received: Family<MessageLabels, Counter>,
}

pub fn setup() -> (MetricFamilies, JoinHandle<()>) {
    let mut registry = <Registry>::default();

    let mf = MetricFamilies {
        // messages_sent: Family::<MessageLabels, Counter>::default(),
        // messages_received: Family::<MessageLabels, Counter>::default(),
        messages: Family::<MessageLabels, Counter>::default(),
    };

    // registry.register(
    //     "messages_sent",
    //     "Number of messages sent",
    //     mf.messages_sent.clone(),
    // );
    // registry.register(
    //     "messages_received",
    //     "Number of messages received",
    //     mf.messages_received.clone(),
    // );
    registry.register(
        "messages",
        "Messages sent through system",
        mf.messages.clone(),
    );

    let server = tiny_http::Server::http("0.0.0.0:5050").unwrap();
    let server_handle = tokio::spawn(async move {
        loop {
            match server.recv() {
                Ok(req) => {
                    let mut buffer = String::new();
                    match encode(&mut buffer, &registry) {
                        Ok(_) => {
                            if let Err(e) = req.respond(Response::from_string(buffer)) {
                                eprintln!("[METRICS] Failed responding: {e}");
                            }
                        }
                        Err(e) => eprintln!("[METRICS] Failed encoding: {e}"),
                    };
                }
                Err(e) => eprintln!("[METRICS] Failed receiving: {e}"),
            };
        }
    });

    (mf, server_handle)
}
