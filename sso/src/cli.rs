use crate::{
    AuditBuilder, Driver, DriverError, DriverResult, KeyCreate, KeyWithValue, NotifyActor,
    NotifyActorOptions, Server, ServerOptions, Service, ServiceCreate,
};
use actix_rt::System;

/// CLI options.
#[derive(Debug, Clone)]
pub struct CliOptions {
    notify_threads: usize,
    notify: NotifyActorOptions,
    server_threads: usize,
    server: ServerOptions,
}

impl CliOptions {
    /// Create new options.
    pub fn new(
        notify_threads: usize,
        notify: NotifyActorOptions,
        server_threads: usize,
        server: ServerOptions,
    ) -> Self {
        Self {
            notify_threads,
            notify,
            server_threads,
            server,
        }
    }

    /// Returns number of notify threads.
    pub fn notify_threads(&self) -> usize {
        self.notify_threads
    }

    /// Returns notify options reference.
    pub fn notify(&self) -> &NotifyActorOptions {
        &self.notify
    }

    /// Returns number of server threads.
    pub fn server_threads(&self) -> usize {
        self.server_threads
    }

    /// Returns server options reference.
    pub fn server(&self) -> &ServerOptions {
        &self.server
    }
}

/// CLI functions.
#[derive(Debug)]
pub struct Cli;

impl Cli {
    /// Create an audit builder for local commands.
    pub fn audit_builder() -> AuditBuilder {
        // TODO(refactor): Call into Api from this layer.
        // AuditBuilder::new(AuditMeta::new("cli", "127.0.0.1", None))
        unimplemented!();
    }

    /// Create a root key.
    pub fn create_root_key(driver: Box<dyn Driver>, name: &str) -> DriverResult<KeyWithValue> {
        let create = KeyCreate::root(true, name);
        driver.key_create(&create).map_err(Into::into)
    }

    /// Create a service with service key.
    pub fn create_service_with_key(
        driver: Box<dyn Driver>,
        name: &str,
        url: &str,
        provider_local_url: Option<&str>,
        provider_github_oauth2_url: Option<&str>,
        provider_microsoft_oauth2_url: Option<&str>,
    ) -> DriverResult<(Service, KeyWithValue)> {
        let service_create = ServiceCreate {
            is_enabled: true,
            name: name.to_owned(),
            url: url.to_owned(),
            provider_local_url: provider_local_url.map(|x| x.to_owned()),
            provider_github_oauth2_url: provider_github_oauth2_url.map(|x| x.to_owned()),
            provider_microsoft_oauth2_url: provider_microsoft_oauth2_url.map(|x| x.to_owned()),
        };
        let service = driver.service_create(&service_create)?;
        let key_create = KeyCreate::service(true, name, service.id);
        let key = driver.key_create(&key_create)?;
        Ok((service, key))
    }

    /// Start server.
    /// Starts notify actor and HTTP server.
    pub fn start_server(driver: Box<dyn Driver>, options: CliOptions) -> DriverResult<()> {
        let system = System::new(crate_name!());

        let notify_options = options.notify().clone();
        let notify_addr = NotifyActor::start(options.notify_threads(), notify_options);

        let server_options = options.server().clone();
        let server_notify_addr = notify_addr.clone();
        Server::start(
            options.server_threads(),
            driver,
            server_options,
            server_notify_addr,
        )?;

        system.run().map_err(DriverError::StdIo).map_err(Into::into)
    }
}
