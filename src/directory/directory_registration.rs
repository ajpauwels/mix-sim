use x25519_dalek::PublicKey;

#[derive(Clone, Debug)]
pub struct DirectoryRegistration {
    pub id: String,
    pub pk: PublicKey,
}
