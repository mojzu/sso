use crate::core;
use crate::driver::sqlite::schema::{kv_data, kv_disk, kv_key, kv_version};
use chrono::{DateTime, Utc};

#[derive(Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "kv_disk"]
#[primary_key(disk_id)]
pub struct Disk {
    pub created_at: String,
    pub updated_at: String,
    pub disk_id: String,
    pub disk_name: String,
    pub disk_chunk_size: i64,
    pub disk_version_retention: i64,
    pub disk_duration_retention: i64,
    pub disk_compression: String,
    pub disk_encryption: String,
    pub disk_encryption_data: String,
}

impl From<Disk> for core::Disk {
    fn from(disk: Disk) -> Self {
        let created_at = disk.created_at.parse::<DateTime<Utc>>().unwrap();
        let updated_at = disk.updated_at.parse::<DateTime<Utc>>().unwrap();
        let encryption_data: core::DiskEncryptionData =
            serde_json::from_str(&disk.disk_encryption_data).unwrap();
        core::Disk {
            created_at,
            updated_at,
            id: disk.disk_id,
            name: disk.disk_name,
            options: core::DiskOptions {
                chunk_size: disk.disk_chunk_size,
                version_retention: disk.disk_version_retention,
                duration_retention: disk.disk_duration_retention,
            },
            compression: core::DiskCompression {
                compression: disk.disk_compression,
            },
            encryption: core::DiskEncryption {
                encryption: disk.disk_encryption,
                encryption_data,
            },
        }
    }
}

#[derive(Insertable)]
#[table_name = "kv_disk"]
pub struct DiskInsert<'a> {
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub disk_id: &'a str,
    pub disk_name: &'a str,
    pub disk_chunk_size: i64,
    pub disk_version_retention: i64,
    pub disk_duration_retention: i64,
    pub disk_compression: &'a str,
    pub disk_encryption: &'a str,
    pub disk_encryption_data: &'a str,
}

#[derive(AsChangeset)]
#[table_name = "kv_disk"]
pub struct DiskUpdate<'a> {
    pub updated_at: &'a str,
    pub disk_name: Option<&'a str>,
}

#[derive(Debug, PartialEq, Identifiable, Queryable, Associations)]
#[belongs_to(Disk)]
#[table_name = "kv_key"]
#[primary_key(key_id)]
pub struct Key {
    pub created_at: String,
    pub updated_at: String,
    pub key_id: String,
    pub key_name: String,
    pub disk_id: String,
    pub version_id: Option<String>,
}

impl From<Key> for core::Key {
    fn from(key: Key) -> Self {
        let created_at = key.created_at.parse::<DateTime<Utc>>().unwrap();
        let updated_at = key.updated_at.parse::<DateTime<Utc>>().unwrap();
        core::Key {
            created_at,
            updated_at,
            id: key.key_id,
            name: key.key_name,
            disk_id: key.disk_id,
            version_id: key.version_id,
        }
    }
}

#[derive(Insertable)]
#[table_name = "kv_key"]
pub struct KeyInsert<'a> {
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub key_id: &'a str,
    pub key_name: &'a str,
    pub disk_id: &'a str,
}

#[derive(AsChangeset)]
#[table_name = "kv_key"]
pub struct KeyUpdate<'a> {
    pub updated_at: &'a str,
    pub key_name: Option<&'a str>,
    pub version_id: Option<&'a str>,
}

#[derive(Debug, PartialEq, Identifiable, Queryable, Associations)]
#[belongs_to(Key)]
#[table_name = "kv_version"]
#[primary_key(version_id)]
pub struct Version {
    pub created_at: String,
    pub version_id: String,
    pub version_hash: Vec<u8>,
    pub version_size: i64,
    pub key_id: String,
}

impl From<Version> for core::Version {
    fn from(version: Version) -> Self {
        let created_at = version.created_at.parse::<DateTime<Utc>>().unwrap();
        core::Version {
            created_at,
            id: version.version_id,
            hash: version.version_hash,
            size: version.version_size,
            key_id: version.key_id,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "kv_version"]
pub struct VersionInsert<'a> {
    pub created_at: &'a str,
    pub version_id: &'a str,
    pub version_hash: &'a [u8],
    pub version_size: i64,
    pub key_id: &'a str,
}

#[derive(Debug, PartialEq, Queryable)]
pub struct Data {
    pub data_chunk: i64,
    pub data_value: Vec<u8>,
    pub version_id: String,
}

impl From<Data> for core::Data {
    fn from(data: Data) -> Self {
        core::Data {
            chunk: data.data_chunk,
            value: data.data_value,
            version_id: data.version_id,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "kv_data"]
pub struct DataInsert<'a> {
    pub data_chunk: i64,
    pub data_value: &'a [u8],
    pub version_id: &'a str,
}
