#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::boxed::Box;
#[cfg(feature = "std")]
use std::boxed::Box;

#[derive(Debug)]
pub enum Error {
    Os(i32),
    Simple(ErrorKind),
    SimpleMessage(&'static str),
    #[cfg(feature = "alloc")]
    Custom {
        kind: ErrorKind,
        error: Box<dyn core::error::Error + Send + Sync>,
    },
}

#[derive(Debug)]
pub enum ErrorKind {
    // todo These are placeholder kinds
    /// An error occurred while sending a datagram.
    SendError,
    /// An error occurred while receiving a datagram.
    RecvError,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Os(code) => write!(f, "OS error: {code}"),
            Error::Simple(kind) => write!(f, "{kind}"),
            Error::SimpleMessage(msg) => write!(f, "{msg}"),
            #[cfg(feature = "alloc")]
            Error::Custom { kind, error } => write!(f, "{kind}: {error}"),
        }
    }
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorKind::SendError => write!(f, "Send error"),
            ErrorKind::RecvError => write!(f, "Receive error"),
        }
    }
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            #[cfg(feature = "alloc")]
            Error::Custom { error, .. } => Some(error.as_ref()),
            _ => None,
        }
    }
}
