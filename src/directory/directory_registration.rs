use x25519_dalek::PublicKey;

#[derive(Clone)]
pub struct DirectoryRegistration {
    pub id: String,
    pub pk: PublicKey,
}
