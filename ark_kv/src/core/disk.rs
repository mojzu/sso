use crate::core::{Disk, DiskOptions, DiskStatus, Error};
use crate::driver::Driver;

pub fn list(driver: &dyn Driver) -> Result<Vec<Disk>, Error> {
    let id_list = driver
        .disk_list_where_name_gte("", None, 1024)
        .map_err(Error::Driver)?;
    let mut disk_list: Vec<Disk> = Vec::new();
    for id in id_list.into_iter() {
        let disk = read_by_id(driver, &id)?;
        disk_list.push(disk);
    }
    Ok(disk_list)
}

pub fn status(driver: &dyn Driver, disk: &str) -> Result<DiskStatus, Error> {
    let disk = read_by_name(driver, disk)?;
    driver.disk_status_by_id(&disk.id).map_err(Error::Driver)
}

pub fn create(driver: &dyn Driver, disk: &str, options: &DiskOptions) -> Result<Disk, Error> {
    driver.disk_create(disk, options).map_err(Error::Driver)
}

pub fn read_by_id(driver: &dyn Driver, id: &str) -> Result<Disk, Error> {
    driver
        .disk_read_by_id(&id)
        .map_err(Error::Driver)
        .and_then(|x| x.ok_or_else(|| Error::Unwrap))
}

pub fn read_by_name(driver: &dyn Driver, name: &str) -> Result<Disk, Error> {
    driver
        .disk_read_by_name(name)
        .map_err(Error::Driver)
        .and_then(|x| x.ok_or_else(|| Error::Unwrap))
}

pub fn delete(driver: &dyn Driver, disk: &str) -> Result<usize, Error> {
    let disk = read_by_name(driver, disk)?;
    driver.disk_delete_by_id(&disk.id).map_err(Error::Driver)
}
