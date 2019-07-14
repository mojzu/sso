use crate::core::{DiskEncryption, DiskEncryptionData, Error};
use sodiumoxide::crypto::box_::curve25519xsalsa20poly1305;
use sodiumoxide::randombytes::randombytes;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

// TODO(refactor): Clear sensitive data from memory on drop.

impl DiskEncryption {
    pub fn new(_encryption: i64) -> Self {
        let (pk, sk) = curve25519xsalsa20poly1305::gen_keypair();
        let data = DiskEncryptionData {
            secret_key: sk.0.to_vec(),
            public_key: pk.0.to_vec(),
        };
        DiskEncryption {
            encryption: "curve25519xsalsa20poly1305".to_owned(),
            encryption_data: data,
        }
    }

    pub fn from_string(secret_key: &str) -> Result<Self, Error> {
        serde_json::from_str(secret_key).map_err(|_err| Error::Unwrap)
    }

    pub fn read_from_file(path: &Path) -> Result<Self, Error> {
        if !path.exists() {
            return Err(Error::Unwrap);
        }

        let mut file = File::open(&path).map_err(|_err| Error::Unwrap)?;
        let mut data = String::new();
        file.read_to_string(&mut data)
            .map_err(|_err| Error::Unwrap)?;
        DiskEncryption::from_string(&data)
    }

    pub fn encryption(&self) -> &str {
        &self.encryption
    }

    pub fn secret_key(&self) -> &[u8] {
        &self.encryption_data.secret_key
    }

    pub fn public_key(&self) -> &[u8] {
        &self.encryption_data.public_key
    }

    pub fn to_string(&self) -> Result<String, Error> {
        serde_json::to_string(self).map_err(|_err| Error::Unwrap)
    }

    pub fn write_to_file(&self, path: &Path) -> Result<String, Error> {
        if path.exists() {
            return Err(Error::Unwrap);
        }

        let data = self.to_string()?;
        let mut file = File::create(&path).map_err(|_err| Error::Unwrap)?;
        file.write_all(data.as_bytes())
            .map_err(|_err| Error::Unwrap)?;
        Ok(format!("{}", path.display()))
    }

    pub fn verify(&self) -> Result<bool, Error> {
        let sk = DiskEncryption::secret_key_from_bytes(self.secret_key())?;
        let pk = DiskEncryption::public_key_from_bytes(self.public_key())?;
        let nonce = curve25519xsalsa20poly1305::gen_nonce();
        let seal_data = randombytes(256);
        let ciphertext = curve25519xsalsa20poly1305::seal(&seal_data, &nonce, &pk, &sk);
        let open_data = curve25519xsalsa20poly1305::open(&ciphertext, &nonce, &pk, &sk)
            .map_err(|_err| Error::Unwrap)?;
        Ok(seal_data[..] == open_data[..])
    }

    fn secret_key_from_bytes(bytes: &[u8]) -> Result<curve25519xsalsa20poly1305::SecretKey, Error> {
        curve25519xsalsa20poly1305::SecretKey::from_slice(bytes).ok_or_else(|| Error::Unwrap)
    }

    fn public_key_from_bytes(bytes: &[u8]) -> Result<curve25519xsalsa20poly1305::PublicKey, Error> {
        curve25519xsalsa20poly1305::PublicKey::from_slice(bytes).ok_or_else(|| Error::Unwrap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_secret_key() {
        let secret_key = DiskEncryption::new(1);
        assert_eq!(secret_key.encryption(), "curve25519xsalsa20poly1305");
        assert!(secret_key.secret_key().len() > 0);
        assert!(secret_key.public_key().len() > 0);
    }
}
