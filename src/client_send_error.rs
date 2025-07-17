use std::{error::Error, fmt::Display};

use tokio::sync::mpsc::error::SendError;

use crate::server_command::ServerCommand;

#[derive(Debug)]
pub enum ClientSendError {
    ServerChannelClosed,
    RecipientNotFound,
}

impl Error for ClientSendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for ClientSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientSendError::ServerChannelClosed => {
                write!(f, "server channel closed")
            }
            ClientSendError::RecipientNotFound => {
                write!(f, "recipient did not exist in the address book")
            }
        }
    }
}

impl From<SendError<ServerCommand>> for ClientSendError {
    fn from(_: SendError<ServerCommand>) -> Self {
        ClientSendError::ServerChannelClosed
    }
}
