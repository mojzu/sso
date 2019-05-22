use sodiumoxide::crypto::box_::curve25519xsalsa20poly1305;
use std::path::Path;

/// Core errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// TODO(refactor): Improve error types.
    #[fail(display = "CoreError::Unwrap")]
    Unwrap,
    /// sodiumoxide initialisation error.
    #[fail(display = "SodiumoxideInit::Unwrap")]
    SodiumoxideInit(()),
}

/// Public-key encryption key.
#[derive(Debug, Serialize, Deserialize)]
pub struct Key {
    alg: String,
    sk: Vec<u8>,
    pk: Vec<u8>,
}

impl Key {
    /// Create new key.
    pub fn create() -> Self {
        let (pk, sk) = curve25519xsalsa20poly1305::gen_keypair();
        Key {
            alg: "curve25519xsalsa20poly1305".to_owned(),
            sk: sk.0.to_vec(),
            pk: pk.0.to_vec(),
        }
    }

    /// Return key deserialised from string reference.
    pub fn from_str(input: &str) -> Result<Self, Error> {
        serde_json::from_str(input).map_err(|_err| Error::Unwrap)
    }

    /// Read key from file at path.
    pub fn read_from_file(path: &Path) -> Result<Self, Error> {
        use std::fs::File;
        use std::io::prelude::*;

        if !path.exists() {
            // File does not exist at path.
            return Err(Error::Unwrap);
        }

        let mut file = File::open(&path).map_err(|_err| Error::Unwrap)?;
        let mut data = String::new();
        file.read_to_string(&mut data)
            .map_err(|_err| Error::Unwrap)?;
        Key::from_str(&data)
    }

    /// Return key serialised as string.
    pub fn to_string(&self) -> Result<String, Error> {
        serde_json::to_string(self).map_err(|_err| Error::Unwrap)
    }

    /// Write serialised key to file at path.
    pub fn write_to_file(self, path: &Path) -> Result<Self, Error> {
        use std::fs::File;
        use std::io::prelude::*;

        if path.exists() {
            // File exists, do not overwrite files.
            return Err(Error::Unwrap);
        }

        let data = self.to_string()?;
        let mut file = File::create(&path).map_err(|_err| Error::Unwrap)?;
        file.write_all(data.as_bytes())
            .map_err(|_err| Error::Unwrap)?;
        Ok(self)
    }
}

/// Initialise library.
pub fn init() -> Result<(), Error> {
    sodiumoxide::init().map_err(Error::SodiumoxideInit)
}
