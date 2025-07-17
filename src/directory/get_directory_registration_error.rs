use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum GetDirectoryRegistrationError {
    NotFound,
}

impl Error for GetDirectoryRegistrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for GetDirectoryRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetDirectoryRegistrationError::NotFound => {
                write!(f, "no registration was found at that id in the directory")
            }
        }
    }
}
