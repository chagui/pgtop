use std::io;
use std::sync::mpsc;

use tokio_postgres;

#[derive(Debug)]
pub enum CliError {
    DBError { source: tokio_postgres::Error },
    UIError { source: io::Error },
    PipeError { source: mpsc::RecvError },
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            CliError::DBError { ref source } => write!(f, "{}", source),
            CliError::UIError { ref source } => write!(f, "{}", source),
            CliError::PipeError { ref source } => write!(f, "{}", source),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            CliError::DBError { ref source } => Some(source),
            CliError::UIError { ref source } => Some(source),
            CliError::PipeError { ref source } => Some(source),
        }
    }
}

impl From<tokio_postgres::Error> for CliError {
    fn from(err: tokio_postgres::Error) -> CliError {
        CliError::DBError { source: err }
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::UIError { source: err }
    }
}

impl From<mpsc::RecvError> for CliError {
    fn from(err: mpsc::RecvError) -> CliError {
        CliError::PipeError { source: err }
    }
}
