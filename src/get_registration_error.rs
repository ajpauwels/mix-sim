use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum GetRegistrationError {
    NotFound,
}

impl Error for GetRegistrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for GetRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetRegistrationError::NotFound => {
                write!(f, "no registration was found at that id")
            }
        }
    }
}
