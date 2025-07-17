use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ServerRegistrationError {
    Conflict,
}

impl Error for ServerRegistrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for ServerRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerRegistrationError::Conflict => {
                write!(f, "id is already registered at server")
            }
        }
    }
}
