use std::io;
use std::sync::mpsc;

#[derive(Debug)]
pub enum CliError {
    DB { source: tokio_postgres::Error },
    UI { source: io::Error },
    Pipe { source: mpsc::RecvError },
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            CliError::DB { ref source } => write!(f, "{}", source),
            CliError::UI { ref source } => write!(f, "{}", source),
            CliError::Pipe { ref source } => write!(f, "{}", source),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            CliError::DB { ref source } => Some(source),
            CliError::UI { ref source } => Some(source),
            CliError::Pipe { ref source } => Some(source),
        }
    }
}

impl From<tokio_postgres::Error> for CliError {
    fn from(err: tokio_postgres::Error) -> CliError {
        CliError::DB { source: err }
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::UI { source: err }
    }
}

impl From<mpsc::RecvError> for CliError {
    fn from(err: mpsc::RecvError) -> CliError {
        CliError::Pipe { source: err }
    }
}
