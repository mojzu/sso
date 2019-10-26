use crate::{
    api::AuthProviderOauth2, DriverError, DriverResult, NotifyActorOptionsSmtp, ServerOptionsRustls,
};
use std::str::FromStr;

/// Environment helper functions.
#[derive(Debug)]
pub struct Env;

impl Env {
    /// Read required environment variable string value.
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

    /// Read SMTP environment variables into options.
    /// If no variables are defined, returns None. Else all variables
    /// are required and an error message logged for each missing variable.
    pub fn smtp(
        smtp_host_name: &str,
        smtp_port_name: &str,
        smtp_user_name: &str,
        smtp_password_name: &str,
    ) -> DriverResult<Option<NotifyActorOptionsSmtp>> {
        if Env::has_any_name(&[
            smtp_host_name,
            smtp_port_name,
            smtp_user_name,
            smtp_password_name,
        ]) {
            let smtp_host = Env::string(smtp_host_name)?;
            let smtp_port = Env::value::<u16>(smtp_port_name)?;
            let smtp_user = Env::string(smtp_user_name)?;
            let smtp_password = Env::string(smtp_password_name)?;

            Ok(Some(NotifyActorOptionsSmtp::new(
                smtp_host,
                smtp_port,
                smtp_user,
                smtp_password,
            )))
        } else {
            Ok(None)
        }
    }

    /// Read OAuth2 environment variables into options.
    /// If no variables are defined, returns None. Else all variables
    /// are required and an error message logged for each missing variable.
    pub fn oauth2(
        client_id_name: &str,
        client_secret_name: &str,
    ) -> DriverResult<Option<AuthProviderOauth2>> {
        if Env::has_any_name(&[client_id_name, client_secret_name]) {
            let client_id = Env::string(client_id_name)?;
            let client_secret = Env::string(client_secret_name)?;

            Ok(Some(AuthProviderOauth2::new(client_id, client_secret)))
        } else {
            Ok(None)
        }
    }

    /// Read Rustls environment variables into options.
    /// If no variables are defined, returns None. Else all variables
    /// are required and an error message logged for each missing variable.
    pub fn rustls(
        crt_pem_name: &str,
        key_pem_name: &str,
        client_pem_name: &str,
    ) -> DriverResult<Option<ServerOptionsRustls>> {
        if Env::has_any_name(&[crt_pem_name, key_pem_name, client_pem_name]) {
            let crt_pem = Env::string(crt_pem_name)?;
            let key_pem = Env::string(key_pem_name)?;
            let client_pem = Env::string_opt(client_pem_name);

            Ok(Some(ServerOptionsRustls::new(crt_pem, key_pem, client_pem)))
        } else {
            Ok(None)
        }
    }
}
