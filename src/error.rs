use quick_error::quick_error;
use std::io;
use wgpu::SurfaceError;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        IO(err: String) {
            display("{}", err)
        }
        Internal(err: String) {
            display("{}", err)
        }
        GraphicsAPI(err: String) {
            display("{}", err)
        }
        ContextLost(err: String) {
            display("{}", err)
        }
        OutOfMemory(err: String) {
            display("{}", err)
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IO(format!("{}", error))
    }
}

impl From<SurfaceError> for Error {
    fn from(error: SurfaceError) -> Self {
        match error {
            SurfaceError::Lost => Error::ContextLost("Lost".to_string()),
            SurfaceError::OutOfMemory => Error::OutOfMemory("Out of memory".to_string()),
            _ => Error::GraphicsAPI(format!("{}", error)),
        }
    }
}

/// The result type used in this crate.
pub type Result<T> = std::result::Result<T, Error>;
