use std::collections::{hash_map::Entry, HashMap};

use tokio::sync::mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender};

use crate::{
    client::ClientCommand, server::ServerCommand, server::ServerRegistration,
    server::ServerRegistrationError,
};

pub struct Server {
    server_tx: MpscSender<ServerCommand>,
    server_rx: MpscReceiver<ServerCommand>,
    registrations: HashMap<String, ServerRegistration>,
}

impl Server {
    pub fn new(buffer_size: usize) -> Self {
        let (server_tx, server_rx) = mpsc::channel::<ServerCommand>(buffer_size);
        Self {
            server_tx,
            server_rx,
            registrations: HashMap::new(),
        }
    }

    pub async fn listen(&mut self) {
        println!("[SERVER] Starting listening");
        while let Some(cmd) = self.server_rx.recv().await {
            match cmd {
                ServerCommand::Register(registration, response_tx) => {
                    match self.registrations.entry(registration.id.clone()) {
                        Entry::Occupied(oe) => {
                            println!("[SERVER] Client already registered at id \"{}\"", oe.key());
                            if let Err(e) = response_tx
                                .send(Err(ServerRegistrationError::Conflict))
                                .await
                            {
                                eprintln!("[SERVER] Failed to notify client that id \"{}\" is already registered: {e}", oe.key());
                            }
                        }
                        Entry::Vacant(ve) => {
                            let oe = ve.insert_entry(registration);
                            println!("[SERVER] Client with id \"{}\" registered", oe.key());
                            if let Err(e) = response_tx.send(Ok(())).await {
                                eprintln!("[SERVER] Failed to notify client that id \"{}\" was successfully registered: {e}", oe.key());
                            }
                        }
                    }
                }
                ServerCommand::Send(msg) => match self.registrations.get(msg.to()) {
                    Some(registration) => match registration.tx {
                        Some(ref tx) => {
                            if let Err(e) = tx.send(ClientCommand::ReceiveMessage(msg)).await {
                                eprintln!("[SERVER] Could not forward message: {e}");
                            }
                        }
                        None => {
                            eprintln!("[SERVER] Could not forward message: client is unavailable");
                        }
                    },
                    None => {
                        println!("[SERVER] No client registered at id \"{}\"", msg.to());
                    }
                },
            }
        }
    }

    pub fn get_tx(&self) -> MpscSender<ServerCommand> {
        self.server_tx.clone()
    }
}
