use err_derive::Error;

use std::io;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "Socket Binding Error")]
    SocketBindingError(#[source] io::Error),
}
