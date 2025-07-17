use std::{collections::HashMap, time::Duration};

use tokio::{
    sync::mpsc::{self, Sender as MpscSender},
    time::sleep,
};

use crate::{
    client_command::ClientCommand, client_send_error::ClientSendError,
    directory_registration::DirectoryRegistration,
};

pub struct User {
    address_book: HashMap<String, DirectoryRegistration>,
    client_tx: MpscSender<ClientCommand>,
    directory_tx: MpscSender<DirectoryCommand>,
}

impl User {
    pub fn new(client_tx: MpscSender<ClientCommand>) -> Self {
        Self {
            address_book: HashMap::new(),
            client_tx,
        }
    }

    pub async fn send_loop(&self, to: &str, body: &str, interval_millis: u64) {
        loop {
            let (response_tx, mut response_rx) = mpsc::channel::<Result<(), ClientSendError>>(1);
            let cmd = ClientCommand::Send(to.to_owned(), body.to_owned(), response_tx);
            if let Err(e) = self.client_tx.send(cmd).await {
                eprintln!("[USER] Failed instructing client to send message to \"{to}\": {e}",);
            }
            match response_rx.recv().await {
                Some(Err(e)) => {
                    eprintln!("[USER] Client failed to send message to \"{to}\": {e}");
                }
                None => {
                    eprintln!("[USER] Send response channel closed before receiving anything");
                }
                _ => {}
            }
            sleep(Duration::from_millis(interval_millis)).await;
        }
    }
}
