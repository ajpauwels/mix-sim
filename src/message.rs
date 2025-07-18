use std::fmt::Display;

use sphinx_packet::SphinxPacket;

pub enum Payload {
    String(String),
    SphinxPacket(SphinxPacket),
}

pub struct Message {
    to: String,
    from: String,
    body: Payload,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.body {
            Payload::String(s) => {
                write!(f, "to: {}, from: {}, {}", &self.to, &self.from, &s)
            }
            Payload::SphinxPacket(_) => {
                write!(f, "to: {}, from: {}, <sphinx packet>", &self.to, &self.from)
            }
        }
    }
}

impl Message {
    pub fn new(to: &str, from: &str, body: &str) -> Self {
        Message {
            to: to.to_owned(),
            from: from.to_owned(),
            body: Payload::String(body.to_owned()),
        }
    }

    pub fn to(&self) -> &str {
        &self.to
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn body(&self) -> &Payload {
        &self.body
    }
}
