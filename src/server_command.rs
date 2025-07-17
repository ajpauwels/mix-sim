use tokio::sync::mpsc::Sender as MpscSender;

use crate::{
    message::Message, registration_error::RegistrationError,
    server_registration::ServerRegistration,
};

pub enum ServerCommand {
    Register(ServerRegistration, MpscSender<Option<RegistrationError>>),
    // GetRegistration(
    //     String,
    //     MpscSender<Result<Registration, GetRegistrationError>>,
    // ),
    Send(Message),
}
