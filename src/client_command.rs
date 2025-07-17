use tokio::sync::mpsc::Sender as MpscSender;

use crate::{client_send_error::ClientSendError, message::Message};

pub enum ClientCommand {
    ReceiveMessage(Message),
    Send(String, String, MpscSender<Result<(), ClientSendError>>),
}
