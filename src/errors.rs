// src/errors.rs

// dependencies
use std::fmt;

// struct type to represent an error from the static file server
#[derive(Debug)]
pub enum ServeError {
    NotFound,
    Io(std::io::Error),
}

// implement the Display trait for the ServeError type
impl fmt::Display for ServeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServeError::NotFound => write!(f, "File not found"),
            ServeError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

// implement the Error trait for the ServeError type
impl std::error::Error for ServeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ServeError::Io(err) => Some(err),
            _ => None,
        }
    }
}