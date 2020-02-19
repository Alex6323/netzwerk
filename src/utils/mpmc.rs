pub use mpsc::Sender;

use futures::channel::mpsc;
use futures::SinkExt;
use futures::FutureExt;
use futures::select;

use async_std::{
    prelude::*,
    task,
};

/// Creates a bounded mpmc channel.
pub fn bounded<T>(capacity: usize) -> (mpsc::Sender<T>, Receiver<T>)
where
    T: Clone + Send + 'static,
{
    let (sender, internal) = mpsc::channel(capacity);
    let (add_sender_tx, add_sender_rx) = mpsc::channel(1);
    let (rmv_sender_tx, rmv_sender_rx) = mpsc::channel(1);

    task::spawn(upd_sender_actor(add_sender_rx, rmv_sender_rx, internal));

    let receiver = Receiver {
        internal,
        capacity,
        add_sender_tx,
    };

    (sender, receiver)
}

async fn upd_sender_actor<T>(
    mut add_sender_rx: mpsc::Receiver<mpsc::Sender<T>>,
    mut rmv_sender_rx: mpsc::Receiver<usize>,
    mut messages_rx: mpsc::Receiver<T>)
where
    T: Clone + Send + 'static
{
    let mut senders = vec![];

    loop {
        select! {

            add_sender_result = add_sender_rx.next().fuse() => match add_sender_result {
                Some(sender) => senders.push(sender),
                None => (),
            },

            rmv_sender_result = rmv_sender_rx.next().fuse() => match rmv_sender_result {
                Some(index) => {
                    senders.remove(index);

                    if senders.is_empty() {
                        break;
                    }
                },
                None => (),
            },

            message_result = messages_rx.next().fuse() => match message_result {
                Some(message) => {
                    for sender in &senders {
                        task::spawn(sender.send(message.clone()));
                    }
                },
                None => (),
            }
        }
    }
}

pub struct Receiver<T>
where
    T: Clone + Send + 'static
{
    internal: mpsc::Receiver<T>,
    capacity: usize,
    add_sender_tx: mpsc::Sender<mpsc::Sender<T>>, //a sender for senders, inception style ;)
}

impl<T> Receiver<T>
where
    T: Clone + Send + 'static
{
    pub async fn next(&mut self) -> Option<T> {
        self.internal.next().await
    }
}

/// Cloning a receiver/consumer means creating a new mpsc channel.
impl<T> Clone for Receiver<T>
where
    T: Clone + Send + 'static
{
    fn clone(&self) -> Self {

        // NOTE: since we cannot actually clone the receiver/consumer, we create a new channel
        // and add the corresponding sender to a list, which we use to iterate and ensure a
        // message gets sent by all senders.
        let (sender, internal) = mpsc::channel(self.capacity);

        task::spawn(self.add_sender_tx.send(sender));

        Self {
            capacity: self.capacity,
            add_sender_tx: self.add_sender_tx.clone(),
        }
    }
}

/// Droppin a receiver/consumer means removing an mpsc channel.
impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {

    }
}


/*
pub async fn unbounded<T>() -> (Sender<T>, Receiver<T>) where T: 'static + std::marker::Send + std::marker::Sync + std::clone::Clone {

    let (incoming_msg_sender, incoming_msg_receiver) = mpsc::unbounded();

    let registered_senders: Arc<Mutex<Vec<UnboundedSender<T>>>> = Arc::new(Mutex::new(Vec::new()));

    task::spawn(receive(incoming_msg_receiver, registered_senders.clone()));

    (Sender::new(incoming_msg_sender), Receiver::new(registered_senders).await)

}

///
async fn receive<T>(mut incoming_messages_receiver: UnboundedReceiver<T>, registered_senders: Arc<Mutex<Vec<UnboundedSender<T>>>>) where T: std::clone::Clone {

    while let Some(event) = incoming_messages_receiver.next().await {
        let set = &*registered_senders.lock().await;
        for sender in set {
            let mut sender: &UnboundedSender<T> = sender;
            sender.send(event.clone()).await;
        }
    }

}
*/

/*
pub struct Sender<T> {
    sender: mpsc::Sender<T>
}

impl<T> Sender<T> {

    fn new(sender: Sender<T>) -> Self {
        Self {
            sender
        }
    }

    pub async fn send(&mut self, data: T) -> Result<(), mpsc::SendError> {
        self.sender.send(data).await
    }

}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self { sender: self.sender.clone() }
    }
}
*/

/*
pub struct Receiver<T> {
    receiver: mpsc::Receiver<T>,
    registered_senders: Arc<Mutex<Vec<UnboundedSender<T>>>>
}

impl<T> Receiver<T> {

    async fn new(registered_senders: Arc<Mutex<Vec<UnboundedSender<T>>>>) -> Self {
        let (sender, receiver) = mpsc::unbounded::<T>();
        let locked_vec = &mut  *registered_senders.lock().await;
        locked_vec.push(sender);
        Self {receiver, registered_senders: Arc::clone(&registered_senders)}
    }

    pub async fn next(&mut self) -> Option<T> {
        self.receiver.next().await
    }

    pub async fn async_clone(&mut self) -> Self {
        let (sender, receiver) = mpsc::unbounded::<T>();
        let locked_vec = &mut  self.registered_senders.lock().await;
        locked_vec.push(sender);
        Self {receiver, registered_senders: Arc::clone(&self.registered_senders)}
    }

}
*/