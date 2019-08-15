use crate::core::Error;
use crate::driver::Driver;

pub fn mount(_driver: &dyn Driver, _disk: &str, _mountpoint: &str) -> Result<(), Error> {
    unimplemented!();
}

// use crate::Driver;
// use fuse::{ReplyAttr, Request};
// use libc::ENOENT;

// const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

// pub fn mount<D: Driver>(driver: &D, disk_name: &str, mountpoint: &str) -> i32 {
//     use std::ffi::OsStr;

//     let fs = crate::filesystem::Fs::new(driver, disk_name.to_owned());
//     let mountpoint = std::path::Path::new(mountpoint);
//     let options = ["-o", "ro", "-o", "fsname=ark"]
//         .iter()
//         .map(|o| o.as_ref())
//         .collect::<Vec<&OsStr>>();
//     fuse::mount(fs, &mountpoint, &options).expect("Failed to mount filesystem");
//     0
// }

// pub struct Fs<'a, D: Driver> {
//     _driver: &'a D,
//     _disk_name: String,
// }

// impl<'a, D: Driver> Fs<'a, D> {
//     pub fn new(driver: &D, disk_name: String) -> Self {
//         Fs {
//             _driver: driver,
//             _disk_name: disk_name,
//         }
//     }
// }

// impl<'a, D: Driver> fuse::Filesystem for Fs<'a, D> {
//     fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
//         println!("ino = {}", ino);
//         reply.error(ENOENT);
//     }
// }
