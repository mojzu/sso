pub mod data;
pub mod disk;
pub mod disk_encryption;
pub mod key;
pub mod version;
// TODO(refactor): Feature flag for fuse support.
pub mod fuse;

use crate::driver::{Driver, Error as DriverError};
use chrono::{DateTime, Utc};

// TODO(feature): Read protect/write protect disks.
// TODO(feature): Symmetric encryption key option.
// TODO(feature): Improve types, support types of encryption.
// TODO(feature): Compression type per key.
// TODO(feature): Implement file driver (mz_ota support?).
// TODO(feature): Use files instead of buffers depending on file size, guess input size for buffers.
// TODO(feature): Key read options with check flags.
// TODO(feature): FUSE filesystem support.
// TODO(feature): Make secret key optional for verify.
// TODO(feature): Flag to enable/disable modified time check.
// TODO(feature): Configurable chunk size, compression.
// TODO(feature): Data deduplication support/testing?
// TODO(feature): Windows support/testing, use features.
// TODO(feature): Remote backup/synchronisation of volumes.
// TODO(feature): Application logs and API.
// TODO(feature): Read to directory datetime option for point in time recovery.
// TODO(feature): HTTP server interface, server read-only files from volumes.
// TODO(refactor): Improve CLI output, formatting, progress for read/write disk.

/// Core errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// TODO(refactor): Clean up errors.
    #[fail(display = "CoreError::Unwrap")]
    Unwrap,
    /// Driver error wrapper.
    #[fail(display = "CoreError::Driver {}", _0)]
    Driver(#[fail(cause)] DriverError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub disk_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskEncryptionData {
    secret_key: Vec<u8>,
    public_key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskEncryption {
    encryption: String,
    encryption_data: DiskEncryptionData,
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
pub struct DiskStatus {
    pub id: String,
    pub name: String,
    pub key_count: i64,
    pub total_size: i64,
    // TODO(feature): Compressed size status.
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStatus {
    pub id: String,
    pub name: String,
    pub version_count: i64,
    pub total_size: i64,
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

pub fn status(driver: &dyn Driver) -> Result<Status, Error> {
    driver.status().map_err(Error::Driver)
}

pub fn poll(driver: &dyn Driver, vacuum: bool) -> Result<(), Error> {
    let disks = self::disk::list(driver)?;
    for (_i, disk) in disks.into_iter().enumerate() {
        if disk.options.version_retention > 0 {
            let keys = self::key::list(driver, &disk)?;

            for (_j, key) in keys.into_iter().enumerate() {
                let versions = self::version::list(driver, &key)?;

                if versions.len() as i64 > disk.options.version_retention {
                    for version in &versions[disk.options.version_retention as usize..] {
                        if disk.options.duration_retention > -1 {
                            let now = Utc::now().timestamp();
                            let duration_compare = now - disk.options.duration_retention;
                            let created_at = version.created_at.timestamp();

                            if duration_compare < created_at {
                                continue;
                            }
                        }

                        self::version::delete_by_id(driver, &version.id)?;
                    }
                }
            }
        }
    }

    if vacuum {
        driver.vacuum().map_err(Error::Driver)?;
    }
    Ok(())
}
