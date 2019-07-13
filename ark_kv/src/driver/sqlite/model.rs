use crate::driver::sqlite::schema::{kv_data, kv_disk, kv_key, kv_version};

#[derive(Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "kv_disk"]
#[primary_key(disk_id)]
pub struct Disk {
    pub created_at: String,
    pub updated_at: String,
    pub disk_id: String,
    pub disk_name: String,
    pub disk_chunk_size: i64,
    pub disk_compression: i64,
    pub disk_encryption: i64,
    pub disk_secret_key: Vec<u8>,
    pub disk_public_key: Vec<u8>,
    pub disk_version_retention: i64,
    pub disk_duration_retention: i64,
}

#[derive(Insertable)]
#[table_name = "kv_disk"]
pub struct DiskInsert<'a> {
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub disk_id: &'a str,
    pub disk_name: &'a str,
    pub disk_chunk_size: i64,
    pub disk_compression: i64,
    pub disk_encryption: i64,
    pub disk_secret_key: &'a [u8],
    pub disk_public_key: &'a [u8],
    pub disk_version_retention: i64,
    pub disk_duration_retention: i64,
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
    pub version_compressed_size: i64,
    pub key_id: String,
}

#[derive(Debug, Insertable)]
#[table_name = "kv_version"]
pub struct VersionInsert<'a> {
    pub created_at: &'a str,
    pub version_id: &'a str,
    pub version_hash: &'a [u8],
    pub version_size: i64,
    pub version_compressed_size: i64,
    pub key_id: &'a str,
}

#[derive(Debug, PartialEq, Queryable)]
pub struct Data {
    pub data_chunk: i64,
    pub data_value: Vec<u8>,
    pub version_id: String,
}

#[derive(Debug, Insertable)]
#[table_name = "kv_data"]
pub struct DataInsert<'a> {
    pub data_chunk: i64,
    pub data_value: &'a [u8],
    pub version_id: &'a str,
}
