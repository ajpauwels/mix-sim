use std::{
    collections::{hash_map::Entry, HashMap},
    time::Duration,
};

use prometheus_client::metrics::{counter::Counter, family::Family};
use rand::prelude::*;
use sphinx_packet::{
    header::delays,
    route::{Destination, DestinationAddressBytes, Node, NodeAddressBytes},
    ProcessedPacketData, SphinxPacket,
};
use tokio::{
    sync::mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender},
    time::sleep,
};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::{
    bytes::{bytes_to_string_truncate_zeroes, str_to_byte_array_32},
    client::{ClientCommand, ClientSendError},
    directory::{
        DirectoryCommand, DirectoryRegistration, DirectoryRegistrationError,
        GetDirectoryRegistrationError,
    },
    packet::{Message, Packet},
    prometheus::{MessageLabels, MessageStatus, MetricFamilies},
    server::{ServerCommand, ServerRegistration, ServerRegistrationError},
};

pub struct ClientMetrics {
    // messages_sent: Family<MessageLabels, Counter>,
    // messages_received: Family<MessageLabels, Counter>,
    messages: Family<MessageLabels, Counter>,
}

pub struct Client {
    id: String,
    sk: StaticSecret,
    address_book: HashMap<String, DirectoryRegistration>,
    directory_tx: MpscSender<DirectoryCommand>,
    client_tx: MpscSender<ClientCommand>,
    client_rx: MpscReceiver<ClientCommand>,
    metrics: Option<ClientMetrics>,
}

impl Client {
    pub fn new(
        id: &str,
        directory_tx: MpscSender<DirectoryCommand>,
        buffer_size: usize,
        mf: &Option<MetricFamilies>,
    ) -> Self {
        let (client_tx, client_rx) = mpsc::channel::<ClientCommand>(buffer_size);
        let sk = StaticSecret::random();
        Self {
            id: id.to_owned(),
            sk,
            address_book: HashMap::new(),
            directory_tx,
            client_tx,
            client_rx,
            metrics: mf.as_ref().map(|mf| ClientMetrics {
                // messages_sent: mf.messages_sent.clone(),
                // messages_received: mf.messages_received.clone(),
                messages: mf.messages.clone(),
            }),
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
                // Shutdown the client
                ClientCommand::Shutdown => {
                    return;
                }
                // Register user at the directory
                ClientCommand::Register => {
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
                            "[CLIENT][{}] Failed to send registration request: {e}",
                            &self.id
                        );
                        return;
                    }
                    match response_rx.recv().await {
                        Some(Ok(_)) => {
                            println!(
                                "[CLIENT][{}] Successfully registered at directory",
                                &self.id
                            );
                        }
                        Some(Err(e)) => {
                            eprintln!(
                                "[CLIENT][{}] Failed to register at directory: {e}",
                                &self.id
                            );
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
                }
                // Receive a packet from another user
                ClientCommand::ReceivePacket(packet) => {
                    let (packet_id, _, from, sphinx_packet) = packet.take();
                    match sphinx_packet.process(&self.sk) {
                        Ok(packet) => match packet.data {
                            ProcessedPacketData::ForwardHop {
                                next_hop_packet,
                                next_hop_address,
                                delay,
                            } => {
                                let rand_val = rand::random_range(0..100);
                                if rand_val < 70 {
                                    let to = bytes_to_string_truncate_zeroes(
                                        next_hop_address.as_bytes(),
                                    );
                                    println!(
                                        "[CLIENT][{}] Forwarding packet from \"{}\" to \"{}\"",
                                        &self.id, &from, &to
                                    );
                                    let packet = Packet::new_with_id(
                                        &packet_id,
                                        &to,
                                        &self.id,
                                        next_hop_packet,
                                    );
                                    sleep(delay.to_duration()).await;
                                    if let Err(e) =
                                        server_tx.send(ServerCommand::Send(packet)).await
                                    {
                                        eprintln!("[CLIENT][{}] Unable to forward packet received from \"{}\" to \"{}\": {e}", &self.id, &from, &to);
                                    }
                                } else {
                                    eprintln!(
                                        "[CLIENT][{}] Client is unavailable at this time",
                                        &self.id
                                    );
                                }
                            }
                            ProcessedPacketData::FinalHop {
                                destination,
                                identifier: _,
                                payload,
                            } => {
                                let to_addr =
                                    bytes_to_string_truncate_zeroes(destination.as_bytes_ref());
                                let payload_bytes = payload.recover_plaintext().unwrap();
                                if to_addr == self.id {
                                    let message: Message =
                                        serde_yaml::from_slice(&payload_bytes).unwrap();
                                    println!(
                                        "[CLIENT][{}] Received message: {}",
                                        &self.id, message.body,
                                    );
                                    if let Some(metrics) = &self.metrics {
                                        metrics
                                            .messages
                                            .get_or_create(&MessageLabels {
                                                from: message.from.unwrap_or("None".to_owned()),
                                                to: self.id.clone(),
                                                status: MessageStatus::Received,
                                            })
                                            .inc();
                                    }
                                } else {
                                    eprintln!("[CLIENT][{}] Do not support forwarding plaintexts at this time", &self.id);
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!(
                                "[CLIENT][{}] Failed to process Sphinx packet from \"{}\": {e}",
                                &self.id, from
                            );
                        }
                    }
                }
                // Send a message to another user
                ClientCommand::Send(to, body, response_tx) => {
                    while self.address_book.len() < 3 {
                        // Fetch all users from directory
                        let (response_tx, mut response_rx) =
                            mpsc::channel::<HashMap<String, DirectoryRegistration>>(1);
                        let cmd = DirectoryCommand::GetAllRegistrations(response_tx);
                        if let Err(e) = self.directory_tx.send(cmd).await {
                            eprintln!(
                                "[CLIENT][{}] Failed to fetch all users from directory: {e}",
                                &self.id
                            );
                            return;
                        }
                        match response_rx.recv().await {
                            Some(mut address_book) => {
                                address_book.remove(&self.id);
                                self.address_book = address_book;
                            }
                            None => {
                                eprintln!("[CLIENT][{}] Get all registrations response channel closed before receiving anything", &self.id);
                                return;
                            }
                        }
                        sleep(Duration::from_millis(2000)).await;
                    }

                    let mut forward_route_entries = self
                        .address_book
                        .values()
                        .filter(|&entry| entry.id != to && entry.id != self.id)
                        .choose_multiple(&mut rand::rng(), 3);
                    forward_route_entries.shuffle(&mut rand::rng());
                    let first_hop_id = match forward_route_entries.first() {
                        Some(entry) => entry.id.clone(),
                        None => to.clone(),
                    };
                    let mut route_string = forward_route_entries
                        .iter()
                        .fold(String::new(), |acc, entry| {
                            acc + &format!("{} -> ", &entry.id)
                        });
                    route_string.push_str(&to);
                    println!(
                        "[CLIENT][{}] Sending message through: {route_string}",
                        &self.id
                    );
                    let mut forward_route = forward_route_entries
                        .iter()
                        .map(|entry| {
                            Node::new(
                                NodeAddressBytes::from_bytes(str_to_byte_array_32(&entry.id)),
                                entry.pk,
                            )
                        })
                        .collect::<Vec<Node>>();
                    match self.address_book.entry(to) {
                        Entry::Occupied(oe) => {
                            let to = oe.key();
                            let destination = Destination::new(
                                DestinationAddressBytes::from_bytes(str_to_byte_array_32(to)),
                                [0u8; 16],
                            );
                            // let sender = Destination::new(
                            //     DestinationAddressBytes::from_bytes(str_to_byte_array_32(&self.id)),
                            //     [0u8; 16],
                            // );
                            forward_route.push(Node::new(
                                NodeAddressBytes::from_bytes(str_to_byte_array_32(to)),
                                oe.get().pk,
                            ));
                            let message = Message {
                                from: Some(self.id.clone()),
                                body,
                            };
                            let message_yaml = serde_yaml::to_string(&message).unwrap();
                            let body_bytes = message_yaml.as_bytes();
                            let average_delay = Duration::from_secs(1);
                            let delays = delays::generate_from_average_duration(
                                forward_route.len(),
                                average_delay,
                            );
                            match SphinxPacket::new(
                                body_bytes.to_vec(),
                                &forward_route,
                                &destination,
                                &delays,
                            ) {
                                Ok(sphinx_packet) => {
                                    let packet =
                                        Packet::new(&first_hop_id, &self.id, sphinx_packet);
                                    // let packet_id = packet.id().to_owned();
                                    let cmd = ServerCommand::Send(packet);
                                    let send_response =
                                        server_tx.send(cmd).await.map_err(ClientSendError::from);
                                    if let Err(e) = &send_response {
                                        eprintln!(
                                            "[CLIENT][{}] Failed sending message to \"{to}\": {e}",
                                            &self.id
                                        );
                                    } else if let Some(metrics) = &self.metrics {
                                        metrics
                                            .messages
                                            .get_or_create(&MessageLabels {
                                                from: self.id.clone(),
                                                to: to.to_owned(),
                                                status: MessageStatus::Sent,
                                            })
                                            .inc();
                                    }
                                    if let Err(e) = response_tx.send(send_response).await {
                                        eprintln!(
                                        "[CLIENT][{}] Failed to respond to request to send message to \"{}\": {e}",
                                        &self.id, &to,
                                    );
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                    "[CLIENT][{}] Failed to construct Sphinx packet to \"{to}\": {e}",
                                    &self.id
                                );
                                }
                            }
                        }
                        Entry::Vacant(ve) => {
                            println!(
                            "[CLIENT][{}] User \"{}\" was not in address book, attempting to fetch from directory",
                            &self.id,
                            ve.key(),
                        );

                            let to = ve.key().to_owned();
                            let (dir_response_tx, mut dir_response_rx) =
                                mpsc::channel::<
                                    Result<DirectoryRegistration, GetDirectoryRegistrationError>,
                                >(1);
                            let cmd =
                                DirectoryCommand::GetRegistration(to.to_owned(), dir_response_tx);
                            if let Err(e) = self.directory_tx.send(cmd).await {
                                eprintln!("[CLIENT][{}] Failed sending get directory registration request for id \"{to}\": {e}", &self.id)
                            }
                            match dir_response_rx.recv().await {
                                Some(Ok(registration)) => {
                                    ve.insert(registration);
                                    if let Err(e) = self.client_tx.send(ClientCommand::Send(to.to_owned(), body, response_tx)).await {
                                        eprintln!("[CLIENT][{}] Failed forwarding send request after fetching missing user from directory: {e}", &self.id);
                                    }
                                }
                                Some(Err(e)) => {
                                    eprintln!("[CLIENT][{}] Failed fetching directory entry for user with id \"{to}\": {e}", &self.id);
                                }
                                None => eprintln!("[CLIENT][{}] Response channel closed before receiving directory entry for user with id \"{to}\"", &self.id),
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_tx(&self) -> MpscSender<ClientCommand> {
        self.client_tx.clone()
    }
}
