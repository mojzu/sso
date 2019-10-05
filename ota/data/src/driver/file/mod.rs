//! # File Driver
use crate::driver::{Driver, Error};

#[derive(Clone)]
pub struct FileDriver {
    file: String,
}

impl FileDriver {
    pub fn initialise(file: &str) -> Result<Self, Error> {
        let driver = FileDriver {
            file: file.to_owned(),
        };
        Ok(driver)
    }
}

impl Driver for FileDriver {
    fn box_clone(&self) -> Box<Driver> {
        Box::new((*self).clone())
    }
}
