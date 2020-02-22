use crate::{DriverError, DriverResult};
use std::str::FromStr;

/// Environment variables.
#[derive(Debug)]
pub struct Env;

impl Env {
    /// Read required environment variable string value.
    ///
    /// Logs an error message in case of error.
    pub fn string(name: &str) -> DriverResult<String> {
        std::env::var(name).map_err(|err| {
            error!("{} is undefined, required ({})", name, err);
            DriverError::StdEnvVar(err)
        })
    }

    /// Read optional environment variable string value.
    pub fn string_opt(name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    /// Read environment variable value parsed from string.
    ///
    /// Logs an error message in case of error.
    pub fn value<T: FromStr>(name: &str) -> DriverResult<T>
    where
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        let value = std::env::var(name).map_err(|err| {
            error!("{} is undefined, required ({})", name, err);
            DriverError::StdEnvVar(err)
        })?;
        match value.parse::<T>() {
            Ok(x) => Ok(x),
            Err(err) => {
                error!("{} is invalid ({})", name, err);
                Err(DriverError::EnvParse(err.to_string()))
            }
        }
    }

    /// Read optional environment variable value parsed from string.
    ///
    /// Logs an error message in case value is not parsed successfully.
    pub fn value_opt<T: FromStr>(name: &str) -> DriverResult<Option<T>>
    where
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        let value = std::env::var(name).ok();
        if let Some(x) = value {
            match x.parse::<T>() {
                Ok(x) => Ok(Some(x)),
                Err(err) => {
                    error!("{} is invalid ({})", name, err);
                    Err(DriverError::EnvParse(err.to_string()))
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Returns true if any name in string reference array is defined in environment.
    pub fn has_any_name(names: &[&str]) -> bool {
        for name in names.iter() {
            if std::env::var(name).is_ok() {
                return true;
            }
        }
        false
    }
}
