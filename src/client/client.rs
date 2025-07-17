use tokio::sync::mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender};

use crate::{
    client::ClientCommand, client::ClientSendError, message::Message, server::ServerCommand,
    server::ServerRegistration, server::ServerRegistrationError,
};

pub struct Client {
    id: String,
    client_tx: MpscSender<ClientCommand>,
    client_rx: MpscReceiver<ClientCommand>,
}

impl Client {
    pub fn new(id: &str, buffer_size: usize) -> Self {
        let (client_tx, client_rx) = mpsc::channel::<ClientCommand>(buffer_size);
        Self {
            id: id.to_owned(),
            client_tx,
            client_rx,
        }
    }

    pub async fn listen(&mut self, server_tx: MpscSender<ServerCommand>) {
        // Register client at server
        let (response_tx, mut response_rx) =
            mpsc::channel::<Result<(), ServerRegistrationError>>(1);
        let cmd = ServerCommand::Register(
            ServerRegistration {
                id: self.id.clone(),
                tx: Some(self.client_tx.clone()),
            },
            response_tx,
        );
        if let Err(e) = server_tx.send(cmd).await {
            eprintln!(
                "[CLIENT][{}] Failed to send registration request: {e}",
                &self.id
            );
            return;
        }
        match response_rx.recv().await {
            Some(Ok(_)) => {
                println!("[CLIENT][{}] Successfully registered at server", &self.id);
            }
            Some(Err(e)) => {
                eprintln!("[CLIENT][{}] Failed to register at server: {e}", &self.id);
                return;
            }
            None => {
                eprintln!(
                    "[CLIENT][{}] Registration response channel closed before receiving anything",
                    &self.id
                );
                return;
            }
        };

        // Loop listening to incoming commands
        println!("[CLIENT][{}] Starting listening", &self.id);
        while let Some(cmd) = self.client_rx.recv().await {
            match cmd {
                // Receive a message from another user
                ClientCommand::ReceiveMessage(msg) => {
                    println!(
                        "[CLIENT][{}] Received message from \"{}\": {}",
                        &self.id,
                        msg.from(),
                        msg.body()
                    );
                }
                // Send a message to another user
                ClientCommand::Send(to, body, response_tx) => {
                    // if let Some(registration) = self.address_book.get(&to) {
                    let cmd = ServerCommand::Send(Message::new(&to, &self.id, &body));
                    let send_response = server_tx.send(cmd).await.map_err(ClientSendError::from);
                    if let Err(e) = &send_response {
                        eprintln!(
                            "[CLIENT][{}] Failed to send message to \"{}\": {e}",
                            &self.id, &to
                        );
                    }
                    if let Err(e) = response_tx.send(send_response).await {
                        eprintln!(
                                "[CLIENT][{}] Failed to respond to request to send message to \"{}\": {e}",
                                &self.id, &to,
                            );
                    }
                    // } else if let Err(e) = response_tx
                    //     .send(Err(ClientSendError::RecipientNotFound))
                    //     .await
                    // {
                    //     eprintln!(
                    //         "[CLIENT][{}] Failed to respond to request to send message to \"{}\": {e}",
                    //         &self.id, &to,
                    //     );
                    // }
                }
            }
        }
    }

    pub fn get_tx(&self) -> MpscSender<ClientCommand> {
        self.client_tx.clone()
    }
}
