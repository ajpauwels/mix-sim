use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum RegistrationError {
    Conflict,
}

impl Error for RegistrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for RegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistrationError::Conflict => {
                write!(f, "id is already registered at server")
            }
        }
    }
}
