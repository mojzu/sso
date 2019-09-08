use crate::{
    AuditBuilder, AuditMeta, AuthError, AuthResult, ClientActor, ClientActorOptions, Driver, Key,
    NotifyActor, NotifyActorOptions, Server, ServerError, ServerOptions, Service,
};
use actix_rt::System;

/// CLI options.
#[derive(Debug, Clone)]
pub struct CliOptions {
    client: ClientActorOptions,
    notify_threads: usize,
    notify: NotifyActorOptions,
    server_threads: usize,
    server: ServerOptions,
}

impl CliOptions {
    /// Create new options.
    pub fn new(
        client: ClientActorOptions,
        notify_threads: usize,
        notify: NotifyActorOptions,
        server_threads: usize,
        server: ServerOptions,
    ) -> Self {
        Self {
            client,
            notify_threads,
            notify,
            server_threads,
            server,
        }
    }

    /// Returns client options reference.
    pub fn client(&self) -> &ClientActorOptions {
        &self.client
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
pub struct Cli;

impl Cli {
    /// Create an audit builder for local commands.
    pub fn audit_builder() -> AuditBuilder {
        AuditBuilder::new(AuditMeta::new("cli", "127.0.0.1", None))
    }

    /// Create a root key.
    pub fn create_root_key(driver: Box<dyn Driver>, name: &str) -> AuthResult<Key> {
        let mut audit = Cli::audit_builder();
        Key::create_root(driver.as_ref(), &mut audit, true, name).map_err(Into::into)
    }

    /// Delete all root keys.
    pub fn delete_root_keys(driver: Box<dyn Driver>) -> AuthResult<usize> {
        let mut audit = Cli::audit_builder();
        Key::delete_root(driver.as_ref(), &mut audit).map_err(Into::into)
    }

    /// Create a service with service key.
    pub fn create_service_with_key(
        driver: Box<dyn Driver>,
        name: &str,
        url: &str,
    ) -> AuthResult<(Service, Key)> {
        let mut audit = Cli::audit_builder();
        let service = Service::create(driver.as_ref(), &mut audit, true, name, url)
            .map_err(AuthError::Core)?;
        let key = Key::create_service(driver.as_ref(), &mut audit, true, name, service.id)
            .map_err(AuthError::Core)?;
        Ok((service, key))
    }

    /// Start server.
    /// Starts notify and client actors, and HTTP server.
    pub fn start_server(driver: Box<dyn Driver>, options: CliOptions) -> AuthResult<()> {
        let system = System::new(crate_name!());

        let client_options = options.client().clone();
        let client_addr = ClientActor::start(client_options);

        let notify_options = options.notify().clone();
        let notify_addr = NotifyActor::start(options.notify_threads(), notify_options);

        let server_options = options.server().clone();
        let server_notify_addr = notify_addr.clone();
        let server_client_addr = client_addr.clone();
        Server::start(
            options.server_threads(),
            driver,
            server_options,
            server_notify_addr,
            server_client_addr,
        )
        .map_err(AuthError::Server)?;

        system.run().map_err(ServerError::StdIo).map_err(Into::into)
    }
}
