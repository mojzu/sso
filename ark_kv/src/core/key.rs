use crate::core;
use crate::core::disk_encryption::HashWriter;
use crate::core::{Disk, DiskEncryption, Error, Key, KeyStatus, KeyWriteOptions, Version};
use crate::driver::Driver;
use std::io::{Read, Write};

// TODO(refactor): Improve arguments consistency.

pub fn list(driver: &dyn Driver, disk: &Disk) -> Result<Vec<Key>, Error> {
    let id_list = driver
        .key_list_where_name_gte("", None, 65536, &disk.id)
        .map_err(Error::Driver)?;
    let mut key_list: Vec<Key> = Vec::new();
    for id in id_list.into_iter() {
        let key = read_by_id(driver, &id)?;
        key_list.push(key);
    }
    Ok(key_list)
}

pub fn status(driver: &dyn Driver, disk: &Disk, key: &str) -> Result<KeyStatus, Error> {
    let key = read_by_name(driver, disk, key)?;
    driver.key_status_by_id(&key.id).map_err(Error::Driver)
}

pub fn create(driver: &dyn Driver, disk: &Disk, key: &str) -> Result<Key, Error> {
    driver.key_create(key, &disk.id).map_err(Error::Driver)
}

pub fn read_by_id(driver: &dyn Driver, id: &str) -> Result<Key, Error> {
    read_opt_by_id(driver, id).and_then(|x| x.ok_or_else(|| Error::Unwrap))
}

pub fn read_opt_by_id(driver: &dyn Driver, id: &str) -> Result<Option<Key>, Error> {
    driver.key_read_by_id(id).map_err(Error::Driver)
}

pub fn read_by_name(driver: &dyn Driver, disk: &Disk, key: &str) -> Result<Key, Error> {
    read_opt_by_name(driver, disk, key).and_then(|x| x.ok_or_else(|| Error::Unwrap))
}

pub fn read_opt_by_name(driver: &dyn Driver, disk: &Disk, key: &str) -> Result<Option<Key>, Error> {
    driver
        .key_read_by_name(key, &disk.id)
        .map_err(Error::Driver)
}

pub fn update_version(driver: &dyn Driver, key: &Key, version: &Version) -> Result<usize, Error> {
    driver
        .key_update_by_id(&key.id, None, Some(&version.id))
        .map_err(Error::Driver)
}

pub fn delete(driver: &dyn Driver, disk: &Disk, key: &str) -> Result<usize, Error> {
    let key = read_by_name(driver, disk, key)?;
    driver.key_delete_by_id(&key.id).map_err(Error::Driver)
}

pub fn read<W: Write>(
    driver: &dyn Driver,
    disk: &str,
    key: &str,
    disk_encryption: &DiskEncryption,
    output: &mut W,
) -> Result<(Key, Version), Error> {
    use flate2::write::ZlibDecoder;
    use std::io::prelude::*;

    let disk = core::disk::read_by_name(driver, disk)?;
    let key = read_by_name(driver, &disk, key)?;
    let version = core::version::read_by_key(driver, &key)?;

    let precomputed = disk_encryption.precompute_read(&disk.options.encryption)?;

    let writer = HashWriter::new(output);
    let mut decoder = ZlibDecoder::new(writer);
    let mut current_chunk: i64 = 0;
    loop {
        let data_cursor = core::data::read_opt_by_version(driver, &version, current_chunk)?;

        if let Some(value) = data_cursor {
            let decrypted_chunk = DiskEncryption::open_precomputed(&value.value, &precomputed)?;

            decoder.write_all(&decrypted_chunk[..]).unwrap();
            current_chunk = value.chunk + 1;
        } else {
            break;
        }
    }

    let writer = decoder.finish().unwrap();
    let (digest, size) = writer.finalize();
    if digest[..] != version.hash[..] {
        return Err(Error::Unwrap);
    }
    if size != version.size {
        return Err(Error::Unwrap);
    }

    Ok((key, version))
}

pub fn write<R: Read>(
    driver: &dyn Driver,
    disk: &str,
    key_name: &str,
    input: &mut R,
    options: KeyWriteOptions,
) -> Result<(Key, Version), Error> {
    let disk = core::disk::read_by_name(driver, disk)?;
    let key = read_opt_by_name(driver, &disk, key_name)?;
    let key = match key {
        Some(x) => x,
        None => create(driver, &disk, key_name)?,
    };
    let version = core::version::read_opt_by_key(driver, &key)?;

    if options.check_modified_time > 0 {
        let updated_at_timestamp = key.updated_at.timestamp();
        if let Some(version) = &version {
            if updated_at_timestamp > options.check_modified_time {
                return Ok((key, version.clone()));
            }
        }
    }

    let precomputed = disk.options.encryption.precompute_write()?;
    let (hash, compressed, size) = hash_and_compress(input)?;

    if let Some(version) = &version {
        // TODO(refactor): Touch file for hash matches so modified time catches next.
        if options.check_hash && hash == version.hash {
            return Ok((key, version.clone()));
        }
        if options.check_size && size == version.size {
            return Ok((key, version.clone()));
        }
    }

    let version = core::version::create(driver, &key, &hash, size)?;
    for (i, chunk) in compressed
        .chunks(disk.options.chunk_size as usize)
        .enumerate()
    {
        let encrypted_chunk = DiskEncryption::seal_precomputed(chunk, &precomputed)?;
        core::data::create(driver, &version, i as i64, &encrypted_chunk)?;
    }

    update_version(driver, &key, &version)?;
    Ok((key, version))
}

pub fn verify<R: Read>(
    _driver: &dyn Driver,
    _disk: &str,
    _key_name: &str,
    _disk_encryption: &DiskEncryption,
    _input: &mut R,
) -> Result<(), Error> {
    unimplemented!();
}

fn hash_and_compress<R: Read>(input: &mut R) -> Result<(Vec<u8>, Vec<u8>, i64), Error> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::prelude::*;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    let mut writer = HashWriter::new(&mut encoder);
    let mut buf = vec![0u8; 8_388_608 as usize];

    loop {
        let sz = input.read(&mut buf).unwrap();

        if sz == 0 {
            break;
        } else {
            writer.write_all(&buf[..sz]).unwrap();
        }
    }

    let (digest, writer_len) = writer.finalize();
    let encoded = encoder.finish().unwrap();
    Ok((digest, encoded, writer_len))
}
