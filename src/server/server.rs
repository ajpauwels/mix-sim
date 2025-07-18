use std::collections::{
    hash_map::{Entry, OccupiedEntry},
    HashMap,
};

use tokio::sync::mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender};

use crate::{
    client::ClientCommand,
    message::Message,
    server::{ServerCommand, ServerRegistration, ServerRegistrationError},
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

    pub async fn register(
        &mut self,
        registration: ServerRegistration,
    ) -> Result<&ServerRegistration, ServerRegistrationError> {
        match self.registrations.entry(registration.id.clone()) {
            Entry::Occupied(oe) => Err(ServerRegistrationError::Conflict(oe.key().to_owned())),
            Entry::Vacant(ve) => Ok(ve.insert(registration)),
        }
    }

    pub async fn send(&self, msg: Message) {
        match self.registrations.get(msg.to()) {
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
        }
    }

    pub async fn listen(&mut self) {
        println!("[SERVER] Starting listening");
        while let Some(cmd) = self.server_rx.recv().await {
            match cmd {
                ServerCommand::Register(registration, response_tx) => {
                    match self.register(registration).await {
                        Ok(registration) => {
                            println!(
                                "[SERVER] Client with id \"{}\" registered",
                                &registration.id
                            );
                            if let Err(e) = response_tx.send(Ok(())).await {
                                eprintln!("[SERVER] Failed to notify client of successful registration: {e}");
                            }
                        }
                        Err(e) => {
                            eprintln!("[SERVER] Failed to register new client: {e}");
                            if let Err(e) = response_tx.send(Err(e)).await {
                                eprintln!("[SERVER] Failed to notify client that an error was encountered during registration: {e}");
                            }
                        }
                    }
                }
                ServerCommand::Send(msg) => self.send(msg).await,
            }
        }
    }

    pub fn get_tx(&self) -> MpscSender<ServerCommand> {
        self.server_tx.clone()
    }
}
