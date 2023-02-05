use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CarError {
    #[error("parsing the file error: {0}")]
    Parsing(String),

    #[error("invalid file error: {0}")]
    InvalidFile(String),

    #[error("invalid section error: {0}")]
    InvalidSection(String),

    #[error("Io error: {0}")]
    IO(#[from] std::io::Error),

    #[error("too large section error: {0}")]
    TooLargeSection(usize),
}
