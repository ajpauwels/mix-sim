use std::collections::{hash_map::Entry, HashMap, VecDeque};

use tokio::sync::mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender};

pub trait Transporter<T> {
    async fn forward(&mut self, to: String, packet: T);
    fn register(&mut self, id: String, tx: MpscSender<T>) -> Option<Vec<T>>;
    fn deregister(&mut self, id: &str);
}

pub struct InboxTransport<T> {
    inboxes: HashMap<String, Vec<T>>,
    links: HashMap<String, MpscSender<T>>,
    tx: MpscSender<(String, T)>,
    rx: MpscReceiver<(String, T)>,
}

impl<T> InboxTransport<T> {
    fn new(buffer_size: usize) -> Self {
        let (tx, rx) = mpsc::channel::<(String, T)>(buffer_size);
        Self {
            inboxes: HashMap::new(),
            links: HashMap::new(),
            rx,
            tx,
        }
    }

    async fn listen(&mut self) {
        println!("[INBOX] Starting listening");
        while let Some((to, packet)) = self.rx.recv().await {
            self.forward(to, packet).await;
        }
    }
}

impl<T> Transporter<T> for InboxTransport<T> {
    async fn forward(&mut self, to: String, packet: T) {
        // Check if we have a link registered for this id
        match self.links.entry(to) {
            // If we have a link, try and forward the packet along
            // with any packets in the id's inbox; if the link fails,
            // remove it and store the packet in the id's inbox
            Entry::Occupied(oe) => {
                let tx = oe.get();
                let inbox = match self.inboxes.remove(oe.key()) {
                    Some(mut inbox) => {
                        inbox.push(packet);
                        inbox
                    }
                    None => {
                        vec![packet]
                    }
                };
                let mut iter = inbox.into_iter();
                while let Some(packet) = iter.next() {
                    if let Err(e) = tx.send(packet).await {
                        let packet = e.0;
                        let mut unprocessed = iter.collect::<VecDeque<T>>();
                        unprocessed.push_front(packet);
                        self.inboxes.insert(oe.key().to_owned(), unprocessed.into());
                        return;
                    }
                }
            }
            // If we don't have a link, store the packet in the id's
            // inbox
            Entry::Vacant(ve) => match self.inboxes.entry(ve.into_key()) {
                Entry::Occupied(mut oe) => {
                    oe.get_mut().push(packet);
                }
                Entry::Vacant(ve) => {
                    ve.insert(vec![packet]);
                }
            },
        }
    }

    fn register(&mut self, id: String, tx: MpscSender<T>) -> Option<Vec<T>> {
        let inboxes = self.inboxes.remove(&id);
        self.links.insert(id, tx);
        inboxes
    }

    fn deregister(&mut self, id: &str) {
        self.links.remove(id);
    }
}
