use tokio::sync::mpsc::Sender as MpscSender;

use crate::client::ClientCommand;

pub struct ServerRegistration {
    pub id: String,
    pub tx: Option<MpscSender<ClientCommand>>,
}
