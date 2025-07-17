use tokio::sync::mpsc::Sender as MpscSender;

use crate::directory::{
    DirectoryRegistration, DirectoryRegistrationError, GetDirectoryRegistrationError,
};

pub enum DirectoryCommand {
    Register(
        DirectoryRegistration,
        MpscSender<Result<(), DirectoryRegistrationError>>,
    ),
    GetRegistration(
        String,
        MpscSender<Result<DirectoryRegistration, GetDirectoryRegistrationError>>,
    ),
}
