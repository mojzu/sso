use crate::core;
use crate::core::disk_encryption::HashWriter;
use crate::core::{Disk, DiskEncryption, Error, Key, KeyWriteOptions, Version};
use crate::driver::Driver;
use std::io::Read;

pub fn create(driver: &Driver, disk: &Disk, key: &str) -> Result<Key, Error> {
    unimplemented!();
}

pub fn read_opt_by_name(driver: &Driver, disk: &Disk, key: &str) -> Result<Option<Key>, Error> {
    unimplemented!();
}

pub fn update_version(driver: &Driver, key: &Key, version: &Version) -> Result<(), Error> {
    unimplemented!();
}

pub fn write<R: Read>(
    driver: &Driver,
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
    let version = core::version::read_by_key(driver, &key)?;

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
        if options.check_hash {
            if hash == version.hash {
                return Ok((key, version.clone()));
            }
        }
        if options.check_size {
            if size == version.size {
                return Ok((key, version.clone()));
            }
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

fn hash_and_compress<R: Read>(input: &mut R) -> Result<(Vec<u8>, Vec<u8>, i64), Error> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::prelude::*;

    // TODO(refactor): Use files instead of buffers depending on file size.
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    let mut writer = HashWriter::new(&mut encoder);
    let mut buf = vec![0u8; 8388608 as usize];

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
