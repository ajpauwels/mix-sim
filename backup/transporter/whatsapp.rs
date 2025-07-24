use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error,
    fmt::Display,
};

use tokio::sync::mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender};

pub struct WhatsApp {
    id: String,
    transporters: HashMap<String, MpscSender<TransporterCommand<String>>>,
    tx: MpscSender<String>,
    rx: MpscReceiver<String>,
}

impl WhatsApp {
    fn new(id: &str, buffer_size: usize) -> Self {
        let (tx, rx) = mpsc::channel::<String>(buffer_size);
        Self {
            id: id.to_owned(),
            transporters: HashMap::new(),
            rx,
            tx,
        }
    }

    fn get_tx(&self) -> MpscSender<String> {
        self.tx.clone()
    }

    async fn add_transporter(
        &mut self,
        id: &str,
        transporter_tx: MpscSender<TransporterCommand<String>>,
    ) {
        let (response_tx, mut response_rx) =
            mpsc::channel::<Result<(String, MpscSender<String>), TransporterRegistrationError>>(1);
        if let Err(e) = transporter_tx
            .send(TransporterCommand::Register(
                Some(self.id.clone()),
                self.tx.clone(),
                response_tx,
            ))
            .await
        {
            eprintln!("[WHATSAPP] Failed to send registration to transporter: {e}");
            return;
        }
        match response_rx.recv().await {
            Some(Ok((_, tx))) => {
                self.transporters.insert(id.to_owned(), transporter_tx);
                self.tx = tx;
            }
            Some(Err(e)) => {
                eprintln!("[WHATSAPP] Failed to register with transporter: {e}");
            }
            None => {
                eprintln!("[WHATSAPP] Transporter registration response channel closed before receiving anything");
            }
        }
    }

    async fn remove_transporter(&mut self, id: &str) {
        match self.transporters.entry(id.to_owned()) {
            Entry::Occupied(oe) => {
                let (response_tx, mut response_rx) = mpsc::channel::<Option<MpscSender<String>>>(1);
                if let Err(e) = oe
                    .get()
                    .send(TransporterCommand::DeRegister(self.id.clone(), response_tx))
                    .await
                {
                    eprintln!("[WHATSAPP] Failed to send deregistration to transporter: {e}");
                    return;
                }
                match response_rx.recv().await {
                    Some(Some(tx)) => {
                        self.tx = tx;
                        oe.remove();
                    }
                    Some(None) => {
                        eprintln!("[WHATSAPP] Failed to deregister from transporter");
                    }
                    None => {
                        eprintln!("[WHATSAPP] Transporter deregistration response channel closed before receiving anything");
                    }
                }
            }
            Entry::Vacant(_) => {
                eprintln!("[WHATSAPP] Not registered at any transporter with id \"{id}\"");
            }
        }
    }
}

pub enum TransporterCommand<T> {
    Register(
        Option<String>,
        MpscSender<T>,
        MpscSender<Result<(String, MpscSender<T>), TransporterRegistrationError>>,
    ),
    DeRegister(String, MpscSender<Option<MpscSender<T>>>),
}

#[derive(Debug)]
pub enum TransporterRegistrationError {
    Conflict(String),
}

impl Error for TransporterRegistrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for TransporterRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransporterRegistrationError::Conflict(id) => {
                write!(f, "\"{}\" is already registered", &id)
            }
        }
    }
}
