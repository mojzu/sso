mod disk_encryption;

use chrono::{DateTime, Utc};

// TODO(feature): Read protect/write protect disks.
// TODO(feature): Symmetric encryption key option.
// TODO(feature): Improve types, support types of encryption.

/// Core errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// TODO(refactor): Clean up errors.
    #[fail(display = "CoreError::Unwrap")]
    Unwrap,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskOptions {
    pub chunk_size: i64,
    pub version_retention: i64,
    pub duration_retention: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskCompression {
    pub compression: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskEncryptionData {
    secret_key: Vec<u8>,
    public_key: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskEncryption {
    pub encryption: String,
    pub encryption_data: DiskEncryptionData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Disk {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: String,
    pub name: String,
    pub options: DiskOptions,
    pub compression: DiskCompression,
    pub encryption: DiskEncryption,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Key {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: String,
    pub name: String,
    pub disk_id: String,
    pub version_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub created_at: DateTime<Utc>,
    pub id: String,
    pub hash: Vec<u8>,
    pub size: i64,
    pub key_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub chunk: i64,
    pub value: Vec<u8>,
    pub version_id: String,
}
