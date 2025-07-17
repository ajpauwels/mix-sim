use tokio::sync::mpsc::Sender as MpscSender;

use crate::{client::ClientSendError, message::Message};

pub enum ClientCommand {
    ReceiveMessage(Message),
    Send(String, String, MpscSender<Result<(), ClientSendError>>),
}
