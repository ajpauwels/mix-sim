use tokio::sync::mpsc::Sender as MpscSender;

use crate::{message::Message, server::ServerRegistration, server::ServerRegistrationError};

pub enum ServerCommand {
    Register(
        ServerRegistration,
        MpscSender<Result<(), ServerRegistrationError>>,
    ),
    Send(Message),
}
