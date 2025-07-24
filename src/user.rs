use std::time::Duration;

use crate::client::{ClientCommand, ClientSendError};
use tokio::{
    sync::mpsc::{self, Sender as MpscSender},
    time::sleep,
};

pub struct User {
    id: String,
    client_tx: MpscSender<ClientCommand>,
}

impl User {
    pub fn new(id: &str, client_tx: MpscSender<ClientCommand>) -> Self {
        Self {
            id: id.to_owned(),
            client_tx,
        }
    }

    pub async fn send_loop(&mut self, to: &str, body: &str, interval_millis: u64) {
        let cmd = ClientCommand::Register;
        if let Err(e) = self.client_tx.send(cmd).await {
            eprintln!(
                "[USER][{}] Failed instructing client to register in directory: {e}",
                &self.id
            );
            return;
        }

        loop {
            if self.id == "alex" {
                let (response_tx, mut response_rx) =
                    mpsc::channel::<Result<(), ClientSendError>>(1);
                if let Err(e) = self
                    .client_tx
                    .send(ClientCommand::Send(
                        to.to_owned(),
                        body.to_owned(),
                        response_tx,
                    ))
                    .await
                {
                    eprintln!("[CLIENT][{}] Failed forwarding send request after fetching missing user from directory: {e}", &self.id);
                } else {
                    match response_rx.recv().await {
                        Some(Err(e)) => {
                            eprintln!(
                                "[USER][{}] Client failed to send message to \"{to}\": {e}",
                                &self.id
                            );
                        }
                        None => {
                            eprintln!(
                                "[USER][{}] Response channel closed before receiving acknowledgement that message was sent to user with id \"{to}\"",
                                &self.id
                            );
                        }
                        _ => {}
                    }
                }
            }

            sleep(Duration::from_millis(interval_millis)).await;
        }
    }
}
