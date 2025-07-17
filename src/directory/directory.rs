use std::collections::{hash_map::Entry, HashMap};

use tokio::sync::mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender};

use crate::directory::{
    DirectoryCommand, DirectoryRegistration, DirectoryRegistrationError,
    GetDirectoryRegistrationError,
};

pub struct Directory {
    directory_tx: MpscSender<DirectoryCommand>,
    directory_rx: MpscReceiver<DirectoryCommand>,
    registrations: HashMap<String, DirectoryRegistration>,
}

impl Directory {
    pub fn new(buffer_size: usize) -> Self {
        let (directory_tx, directory_rx) = mpsc::channel::<DirectoryCommand>(buffer_size);
        Self {
            directory_tx,
            directory_rx,
            registrations: HashMap::new(),
        }
    }

    pub async fn listen(&mut self) {
        println!("[SERVER] Starting listening");
        while let Some(cmd) = self.directory_rx.recv().await {
            match cmd {
                DirectoryCommand::Register(registration, response_tx) => {
                    match self.registrations.entry(registration.id.clone()) {
                        Entry::Occupied(oe) => {
                            println!(
                                "[DIRECTORY] Registration already exists at id \"{}\"",
                                oe.key()
                            );
                            if let Err(e) = response_tx
                                .send(Err(DirectoryRegistrationError::Conflict))
                                .await
                            {
                                eprintln!("[DIRECTORY] Failed to respond that id \"{}\" is already registered: {e}", oe.key());
                            }
                        }
                        Entry::Vacant(ve) => {
                            let oe = ve.insert_entry(registration);
                            println!(
                                "[DIRECTORY] Registration with id \"{}\" has been added",
                                oe.key()
                            );
                            if let Err(e) = response_tx.send(Ok(())).await {
                                eprintln!("[DIRECTORY] Failed to respond that id \"{}\" was successfully added: {e}", oe.key());
                            }
                        }
                    }
                }
                DirectoryCommand::GetRegistration(id, response_tx) => {
                    match self.registrations.get(&id) {
                        Some(registration) => {
                            if let Err(e) = response_tx
                                .send(Ok(DirectoryRegistration {
                                    id: registration.id.clone(),
                                    pk: registration.pk,
                                }))
                                .await
                            {
                                eprintln!(
                                    "[DIRECTORY] Failed to return registration with id \"{id}\": {e}"
                                );
                            }
                        }
                        None => {
                            if let Err(e) = response_tx
                                .send(Err(GetDirectoryRegistrationError::NotFound))
                                .await
                            {
                                eprintln!("[DIRECTORY] Failed to respond that registration with id \"{id}\" did not exist: {e}");
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_tx(&self) -> MpscSender<DirectoryCommand> {
        self.directory_tx.clone()
    }
}
