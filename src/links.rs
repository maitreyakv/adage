//! TODO: links module documentation

use std::convert::Infallible;

pub trait InputReceiver: Send + 'static {
    type Data;
    type Error: std::error::Error;

    fn try_recv(self) -> impl Future<Output = Result<Self::Data, Self::Error>> + Send;
}

struct EmptyReceiver;
impl InputReceiver for EmptyReceiver {
    type Data = ();
    type Error = Infallible;

    fn try_recv(self) -> impl Future<Output = Result<Self::Data, Self::Error>> + Send {
        std::future::ready(Ok(()))
    }
}
