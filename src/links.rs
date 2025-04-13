//! TODO: links module documentation

use std::convert::Infallible;

use tokio::sync::broadcast::{Receiver, Sender};

pub trait InputReceiver: Send + 'static {
    type Data;
    type Error: std::error::Error;

    fn try_recv(self) -> impl Future<Output = Result<Self::Data, Self::Error>> + Send;
}

pub struct EmptyReceiver;
impl InputReceiver for EmptyReceiver {
    type Data = ();
    type Error = Infallible;

    fn try_recv(self) -> impl Future<Output = Result<Self::Data, Self::Error>> + Send {
        std::future::ready(Ok(()))
    }
}

pub struct Linker<T>(Sender<T>);
impl<T> Linker<T> {
    pub(crate) fn new(sender: Sender<T>) -> Self {
        Self(sender)
    }

    pub fn link(&self) -> Receiver<T> {
        self.0.subscribe()
    }
}
