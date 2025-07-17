use std::fmt::Display;

#[derive(Clone, Debug)]
pub struct Message {
    to: String,
    from: String,
    body: String,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "to: {}, from: {}, {}", &self.to, &self.from, &self.body)
    }
}

impl Message {
    pub fn new(to: &str, from: &str, body: &str) -> Self {
        Message {
            to: to.to_owned(),
            from: from.to_owned(),
            body: body.to_owned(),
        }
    }

    pub fn to(&self) -> &str {
        &self.to
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}
