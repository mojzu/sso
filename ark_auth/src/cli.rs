//! # Command Line Interface
//! Functions for a command line interface and some helpers for integration.
use crate::crate_user_agent;
use crate::driver::Driver;
use crate::notify::NotifyExecutor;
use crate::{core, notify, server};
use actix_rt::System;
use std::str::FromStr;

/// Command line interface errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Environment variable parse error.
    #[fail(display = "CliError::EnvParse {}", _0)]
    EnvParse(String),
    /// Core error wrapper.
    #[fail(display = "CliError::Core {}", _0)]
    Core(#[fail(cause)] core::Error),
    /// Server error wrapper.
    #[fail(display = "CliError::Server {}", _0)]
    Server(#[fail(cause)] server::Error),
    /// Standard environment variable error wrapper.
    #[fail(display = "CliError::StdEnvVar {}", _0)]
    StdEnvVar(#[fail(cause)] std::env::VarError),
}

/// Configuration.
pub struct Configuration {
    notify_threads: usize,
    notify: notify::Configuration,
    server_threads: usize,
    server: server::Configuration,
}

impl Configuration {
    /// Create new configuration.
    pub fn new(
        notify_threads: usize,
        notify: notify::Configuration,
        server_threads: usize,
        server: server::Configuration,
    ) -> Self {
        Self {
            notify_threads,
            notify,
            server_threads,
            server,
        }
    }

    /// Get number of notify threads.
    pub fn notify_threads(&self) -> usize {
        self.notify_threads
    }

    /// Get reference to notify configuration.
    pub fn notify(&self) -> &notify::Configuration {
        &self.notify
    }

    /// Get number of server threads.
    pub fn server_threads(&self) -> usize {
        self.server_threads
    }

    /// Get reference to server configuration.
    pub fn server(&self) -> &server::Configuration {
        &self.server
    }
}

/// Read required environment variable string value.
/// Logs an error message in case of error.
pub fn env_string(name: &str) -> Result<String, Error> {
    std::env::var(name).map_err(|err| {
        error!("{} is undefined, required ({})", name, err);
        Error::StdEnvVar(err)
    })
}

/// Read optional environment variable string value.
pub fn env_string_opt(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Read environment variable value parsed from string.
/// Logs an error message in case of error.
pub fn env_value<T: FromStr>(name: &str) -> Result<T, Error>
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let value = std::env::var(name).map_err(|err| {
        error!("{} is undefined, required ({})", name, err);
        Error::StdEnvVar(err)
    })?;

    match value.parse::<T>() {
        Ok(x) => Ok(x),
        Err(err) => {
            error!("{} is invalid ({})", name, err);
            Err(Error::EnvParse(err.to_string()))
        }
    }
}

/// Read optional environment variable value parsed from string.
/// Logs an error message in case value is not parsed successfully.
pub fn env_value_opt<T: FromStr>(name: &str) -> Result<Option<T>, Error>
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let value = std::env::var(name).ok();
    if let Some(x) = value {
        match x.parse::<T>() {
            Ok(x) => Ok(Some(x)),
            Err(err) => {
                error!("{} is invalid ({})", name, err);
                Err(Error::EnvParse(err.to_string()))
            }
        }
    } else {
        Ok(None)
    }
}

/// Returns true if any name in string reference array is defined in environment.
pub fn env_has_any_name(names: &[&str]) -> bool {
    for name in names.into_iter() {
        if std::env::var(name).is_ok() {
            return true;
        }
    }
    return false;
}

/// Read SMTP environment variables into configuration.
/// If no variables are defined, returns None. Else all variables
/// are required and an error message logged for each missing variable.
pub fn env_smtp(
    smtp_host_name: &str,
    smtp_port_name: &str,
    smtp_user_name: &str,
    smtp_password_name: &str,
) -> Result<Option<notify::ConfigurationSmtp>, Error> {
    if env_has_any_name(&[
        smtp_host_name,
        smtp_port_name,
        smtp_user_name,
        smtp_password_name,
    ]) {
        let smtp_host = env_string(smtp_host_name)?;
        let smtp_port = env_value::<u16>(smtp_port_name)?;
        let smtp_user = env_string(smtp_user_name)?;
        let smtp_password = env_string(smtp_password_name)?;

        Ok(Some(notify::ConfigurationSmtp::new(
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_password,
        )))
    } else {
        Ok(None)
    }
}

/// Read OAuth2 environment variables into configuration.
/// If no variables are defined, returns None. Else all variables
/// are required and an error message logged for each missing variable.
pub fn env_oauth2(
    client_id_name: &str,
    client_secret_name: &str,
    redirect_url_name: &str,
) -> Result<Option<server::ConfigurationProviderOauth2>, Error> {
    if env_has_any_name(&[client_id_name, client_secret_name, redirect_url_name]) {
        let client_id = env_string(client_id_name)?;
        let client_secret = env_string(client_secret_name)?;
        let redirect_url = env_string(redirect_url_name)?;

        Ok(Some(server::ConfigurationProviderOauth2::new(
            client_id,
            client_secret,
            redirect_url,
        )))
    } else {
        Ok(None)
    }
}

/// Read Rustls environment variables into configuration.
/// If no variables are defined, returns None. Else all variables
/// are required and an error message logged for each missing variable.
pub fn env_rustls(
    crt_pem_name: &str,
    key_pem_name: &str,
    client_auth_name: &str,
) -> Result<Option<server::ConfigurationRustls>, Error> {
    if env_has_any_name(&[crt_pem_name, key_pem_name, client_auth_name]) {
        let crt_pem = env_string(crt_pem_name)?;
        let key_pem = env_string(key_pem_name)?;
        let client_auth = env_value::<bool>(client_auth_name)?;

        Ok(Some(server::ConfigurationRustls::new(
            crt_pem,
            key_pem,
            client_auth,
        )))
    } else {
        Ok(None)
    }
}

/// Create an audit builder for local commands.
pub fn audit_builder() -> core::audit::AuditBuilder {
    core::audit::AuditBuilder::new(core::AuditMeta::new(
        crate_user_agent(),
        "127.0.0.1".to_owned(),
        None,
    ))
}

/// Create a root key.
pub fn create_root_key(driver: Box<Driver>, name: &str) -> Result<core::Key, Error> {
    let mut audit = audit_builder();
    core::key::create_root(driver.as_ref(), &mut audit, true, name).map_err(Error::Core)
}

/// Delete all root keys.
pub fn delete_root_keys(driver: Box<Driver>) -> Result<usize, Error> {
    let mut audit = audit_builder();
    core::key::delete_root(driver.as_ref(), &mut audit).map_err(Error::Core)
}

/// Create a service with service key.
pub fn create_service_with_key(
    driver: Box<Driver>,
    name: &str,
    url: &str,
) -> Result<(core::Service, core::Key), Error> {
    let mut audit = audit_builder();
    let service =
        core::service::create(driver.as_ref(), &mut audit, true, name, url).map_err(Error::Core)?;
    let key = core::key::create_service(driver.as_ref(), &mut audit, true, name, &service.id)
        .map_err(Error::Core)?;
    Ok((service, key))
}

/// Start server.
/// Starts notify actor and HTTP server.
pub fn start_server(driver: Box<Driver>, configuration: Configuration) -> Result<(), Error> {
    let system = System::new(crate_name!());

    let notify_configuration = configuration.notify().clone();
    let notify_addr = NotifyExecutor::start(configuration.notify_threads(), notify_configuration);

    let server_configuration = configuration.server().clone();
    let server_notify_addr = notify_addr.clone();
    server::start(
        configuration.server_threads(),
        driver,
        server_configuration,
        server_notify_addr,
    )
    .map_err(Error::Server)?;

    system
        .run()
        .map_err(server::Error::StdIo)
        .map_err(Error::Server)
}
