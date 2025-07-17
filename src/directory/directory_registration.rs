use x25519_dalek::PublicKey;

pub struct DirectoryRegistration {
    pub id: String,
    pub pk: PublicKey,
}
