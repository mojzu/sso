//! # Command Line Interface
//! Functions for a command line interface and some helpers for integration.
use crate::crate_user_agent;
use crate::driver::Driver;
use crate::notify::NotifyExecutor;
use crate::{core, notify, server};
use actix_rt::System;

/// Command line interface errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Core error wrapper.
    #[fail(display = "CliError::Core {}", _0)]
    Core(#[fail(cause)] core::Error),
    /// Server error wrapper.
    #[fail(display = "CliError::Server {}", _0)]
    Server(#[fail(cause)] server::Error),
    /// Standard environment variable error wrapper.
    #[fail(display = "CliError::StdEnvVar {}", _0)]
    StdEnvVar(#[fail(cause)] std::env::VarError),
    /// Standard number parse integer error wrapper.
    #[fail(display = "CliError::StdNumParseInt {}", _0)]
    StdNumParseInt(#[fail(cause)] std::num::ParseIntError),
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
pub fn str_from_env(name: &str) -> Result<String, Error> {
    std::env::var(name).map_err(|err| {
        error!("{} is undefined, required ({})", name, err);
        Error::StdEnvVar(err)
    })
}

/// Read optional environment variable string value.
pub fn opt_str_from_env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Read optional environment variable u32 value.
/// Logs an error message in case value is not a valid unsigned integer.
pub fn opt_u32_from_env(name: &str) -> Result<Option<u32>, Error> {
    let value = std::env::var(name).ok();
    if let Some(x) = value {
        match x.parse::<u32>() {
            Ok(x) => Ok(Some(x)),
            Err(err) => {
                error!("{} is invalid unsigned integer ({})", name, err);
                Err(Error::StdNumParseInt(err))
            }
        }
    } else {
        Ok(None)
    }
}

/// Read SMTP environment variables into configuration.
/// If no variables are defined, returns None. Else all variables
/// are required and an error message logged for each missing variable.
pub fn smtp_from_env(
    smtp_host_name: &str,
    smtp_port_name: &str,
    smtp_user_name: &str,
    smtp_password_name: &str,
) -> Result<Option<notify::ConfigurationSmtp>, Error> {
    let smtp_host = std::env::var(smtp_host_name).map_err(|err| {
        error!("{} is undefined ({})", smtp_host_name, err);
        Error::StdEnvVar(err)
    });
    let smtp_port = std::env::var(smtp_port_name).map_err(|err| {
        error!("{} is undefined ({})", smtp_port_name, err);
        Error::StdEnvVar(err)
    });
    let smtp_user = std::env::var(smtp_user_name).map_err(|err| {
        error!("{} is undefined ({})", smtp_user_name, err);
        Error::StdEnvVar(err)
    });
    let smtp_password = std::env::var(smtp_password_name).map_err(|err| {
        error!("{} is undefined ({})", smtp_password_name, err);
        Error::StdEnvVar(err)
    });
    if smtp_host.is_ok() || smtp_port.is_ok() || smtp_user.is_ok() || smtp_password.is_ok() {
        let smtp_host = smtp_host?;
        let smtp_port = smtp_port?;
        let smtp_user = smtp_user?;
        let smtp_password = smtp_password?;

        match smtp_port.parse::<u16>() {
            Ok(x) => Ok(Some(notify::ConfigurationSmtp::new(
                smtp_host,
                x,
                smtp_user,
                smtp_password,
            ))),
            Err(err) => {
                error!("{} is invalid port number ({})", smtp_port_name, err);
                Err(Error::StdNumParseInt(err))
            }
        }
    } else {
        Ok(None)
    }
}

/// Read OAuth2 environment variables into configuration.
/// If no variables are defined, returns None. Else all variables
/// are required and an error message logged for each missing variable.
pub fn oauth2_from_env(
    client_id_name: &str,
    client_secret_name: &str,
    redirect_url_name: &str,
) -> Result<Option<server::ConfigurationProviderOauth2>, Error> {
    let client_id = std::env::var(client_id_name).map_err(|err| {
        error!("{} is undefined ({})", client_id_name, err);
        Error::StdEnvVar(err)
    });
    let client_secret = std::env::var(client_secret_name).map_err(|err| {
        error!("{} is undefined ({})", client_secret_name, err);
        Error::StdEnvVar(err)
    });
    let redirect_url = std::env::var(redirect_url_name).map_err(|err| {
        error!("{} is undefined ({})", redirect_url_name, err);
        Error::StdEnvVar(err)
    });
    if client_id.is_ok() || client_secret.is_ok() || redirect_url.is_ok() {
        let client_id = client_id?;
        let client_secret = client_secret?;
        let redirect_url = redirect_url?;
        Ok(Some(server::ConfigurationProviderOauth2::new(
            client_id,
            client_secret,
            redirect_url,
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
