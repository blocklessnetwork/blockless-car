use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CarError {
    #[error("parsing the file error: {0}")]
    Parsing(String),
    #[error("Io error: {0}")]
    IO(#[from] std::io::Error),
}
