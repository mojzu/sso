pub mod data;
pub mod disk;
pub mod disk_encryption;
pub mod key;
pub mod version;

use crate::driver;
use chrono::{DateTime, Utc};

// TODO(feature): Read protect/write protect disks.
// TODO(feature): Symmetric encryption key option.
// TODO(feature): Improve types, support types of encryption.
// TODO(feature): Compression type per key.

/// Core errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// TODO(refactor): Clean up errors.
    #[fail(display = "CoreError::Unwrap")]
    Unwrap,
    /// Driver error wrapper.
    #[fail(display = "CoreError::Driver {}", _0)]
    Driver(#[fail(cause)] driver::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskEncryptionData {
    secret_key: Vec<u8>,
    public_key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskEncryption {
    pub encryption: String,
    pub encryption_data: DiskEncryptionData,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct DiskOptions {
    pub chunk_size: i64,
    pub compression: String,
    pub encryption: DiskEncryption,
    pub version_retention: i64,
    pub duration_retention: i64,
}

impl DiskOptions {
    pub fn new(
        chunk_size: i64,
        compression: String,
        encryption: DiskEncryption,
        version_retention: i64,
        duration_retention: i64,
    ) -> Self {
        Self {
            chunk_size,
            compression,
            encryption,
            version_retention,
            duration_retention,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disk {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: String,
    pub name: String,
    pub options: DiskOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Key {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: String,
    pub name: String,
    pub disk_id: String,
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct KeyWriteOptions {
    #[builder(default = "true")]
    pub check_hash: bool,
    #[builder(default = "true")]
    pub check_size: bool,
    #[builder(default = "0")]
    pub check_modified_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub created_at: DateTime<Utc>,
    pub id: String,
    pub hash: Vec<u8>,
    pub size: i64,
    pub key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    pub chunk: i64,
    pub value: Vec<u8>,
    pub version_id: String,
}
