use tokio::sync::mpsc::Sender as MpscSender;

use crate::client_command::ClientCommand;

pub struct ServerRegistration {
    pub id: String,
    pub tx: Option<MpscSender<ClientCommand>>,
}
