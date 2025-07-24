use tokio::sync::mpsc::Sender as MpscSender;

use crate::{packet::Packet, server::ServerRegistration, server::ServerRegistrationError};

pub enum ServerCommand {
    Register(
        ServerRegistration,
        MpscSender<Result<(), ServerRegistrationError>>,
    ),
    Send(Packet),
}
