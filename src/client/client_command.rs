use tokio::sync::mpsc::Sender as MpscSender;

use crate::{client::ClientSendError, packet::Packet};

pub enum ClientCommand {
    Register,
    ReceivePacket(Packet),
    Send(String, String, MpscSender<Result<(), ClientSendError>>),
    Shutdown,
}
