use std::{
    collections::{hash_map::Entry, HashMap},
    time::Duration,
};

use sphinx_packet::{
    header::delays,
    route::{Destination, DestinationAddressBytes, Node, NodeAddressBytes},
    SphinxPacket,
};
use tokio::{
    sync::mpsc::{self, Sender as MpscSender},
    time::sleep,
};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::{
    bytes::str_to_byte_array_32,
    client::{ClientCommand, ClientSendError},
    directory::{
        DirectoryCommand, DirectoryRegistration, DirectoryRegistrationError,
        GetDirectoryRegistrationError,
    },
};

pub struct User {
    id: String,
    sk: StaticSecret,
    address_book: HashMap<String, DirectoryRegistration>,
    client_tx: MpscSender<ClientCommand>,
    directory_tx: MpscSender<DirectoryCommand>,
}

impl User {
    pub fn new(
        id: &str,
        client_tx: MpscSender<ClientCommand>,
        directory_tx: MpscSender<DirectoryCommand>,
    ) -> Self {
        let sk = StaticSecret::random();
        Self {
            id: id.to_owned(),
            sk,
            address_book: HashMap::new(),
            client_tx,
            directory_tx,
        }
    }

    pub async fn send_loop(&mut self, to: &str, body: &str, interval_millis: u64) {
        // Register user at directory
        let (response_tx, mut response_rx) =
            mpsc::channel::<Result<(), DirectoryRegistrationError>>(1);
        let cmd = DirectoryCommand::Register(
            DirectoryRegistration {
                id: self.id.clone(),
                pk: PublicKey::from(&self.sk),
            },
            response_tx,
        );
        if let Err(e) = self.directory_tx.send(cmd).await {
            eprintln!(
                "[USER][{}] Failed to send registration request: {e}",
                &self.id
            );
            return;
        }
        match response_rx.recv().await {
            Some(Ok(_)) => {
                println!("[USER][{}] Successfully registered at directory", &self.id);
            }
            Some(Err(e)) => {
                eprintln!("[USER][{}] Failed to register at directory: {e}", &self.id);
                return;
            }
            None => {
                eprintln!(
                    "[USER][{}] Registration response channel closed before receiving anything",
                    &self.id
                );
                return;
            }
        };

        loop {
            match self.address_book.entry(to.to_owned()) {
                Entry::Occupied(oe) => {
                    let destination = Destination::new(
                        DestinationAddressBytes::from_bytes(str_to_byte_array_32(to)),
                        [0u8; 16],
                    );
                    let sender = Destination::new(
                        DestinationAddressBytes::from_bytes(str_to_byte_array_32(&self.id)),
                        [0u8; 16],
                    );
                    let forward_route = [Node::new(
                        NodeAddressBytes::from_bytes(str_to_byte_array_32(to)),
                        oe.get().pk,
                    )];
                    let body_bytes = body.as_bytes();
                    let average_delay = Duration::from_secs(1);
                    let delays =
                        delays::generate_from_average_duration(forward_route.len(), average_delay);
                    let sphinx_packet = SphinxPacket::new(
                        body_bytes.to_vec(),
                        &forward_route,
                        &destination,
                        &delays,
                    );

                    let (response_tx, mut response_rx) =
                        mpsc::channel::<Result<(), ClientSendError>>(1);
                    let cmd = ClientCommand::Send(to.to_owned(), body.to_owned(), response_tx);
                    if let Err(e) = self.client_tx.send(cmd).await {
                        eprintln!(
                            "[USER][{}] Failed instructing client to send message to \"{to}\": {e}",
                            &self.id
                        );
                    }
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
                Entry::Vacant(ve) => {
                    println!(
                    "[USER][{}] User \"{}\" was not in address book, attempting to fetch from directory",
                    &self.id,
                    to
                );
                    let (response_tx, mut response_rx) = mpsc::channel::<
                        Result<DirectoryRegistration, GetDirectoryRegistrationError>,
                    >(1);
                    let cmd = DirectoryCommand::GetRegistration(to.to_owned(), response_tx);
                    if let Err(e) = self.directory_tx.send(cmd).await {
                        eprintln!("[USER][{}] Failed sending get directory registration request for id \"{to}\": {e}", &self.id)
                    }
                    match response_rx.recv().await {
                        Some(Ok(registration)) => {
                            ve.insert(registration);
                        }
                        Some(Err(e)) => {
                            eprintln!("[USER][{}] Failed fetching directory entry for user with id \"{to}\": {e}", &self.id);
                        }
                        None => eprintln!("[USER][{}] Response channel closed before receiving directory entry for user with id \"{to}\"", &self.id),
                    }
                }
            }

            sleep(Duration::from_millis(interval_millis)).await;
        }
    }
}
