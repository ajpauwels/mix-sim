use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sphinx_packet::SphinxPacket;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Message {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    pub body: String,
}

pub struct Packet {
    id: String,
    to: String,
    from: String,
    body: SphinxPacket,
}

impl Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "to: {}, from: {}", &self.to, &self.from)
    }
}

impl Packet {
    pub fn new(to: &str, from: &str, body: SphinxPacket) -> Self {
        Self::new_with_id(&Uuid::new_v4().to_string(), to, from, body)
    }

    pub fn new_with_id(id: &str, to: &str, from: &str, body: SphinxPacket) -> Self {
        Packet {
            id: id.to_owned(),
            to: to.to_owned(),
            from: from.to_owned(),
            body,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn to(&self) -> &str {
        &self.to
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn body(&self) -> &SphinxPacket {
        &self.body
    }

    pub fn take(self) -> (String, String, String, SphinxPacket) {
        (self.id, self.to, self.from, self.body)
    }
}
