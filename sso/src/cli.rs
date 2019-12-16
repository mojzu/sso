use crate::{AuditBuilder, Driver, DriverResult, KeyCreate, KeyWithValue, Service, ServiceCreate};
use chrono::Duration;

/// CLI options.
#[derive(Debug, Clone)]
pub struct CliOptions {
    task_tick_ms: u64,
    audit_retention: Duration,
}

impl CliOptions {
    /// Create new options.
    pub fn new() -> Self {
        Self {
            audit_retention: Duration::weeks(12),
            task_tick_ms: 1000,
        }
    }

    /// Returns task tick interval in milliseconds.
    pub fn task_tick_ms(&self) -> u64 {
        self.task_tick_ms
    }

    /// Returns audit log retention duration.
    pub fn audit_retention(&self) -> &Duration {
        &self.audit_retention
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
        user_allow_register: Option<&str>,
        user_email_text: Option<&str>,
        provider_local_url: Option<&str>,
        provider_github_oauth2_url: Option<&str>,
        provider_microsoft_oauth2_url: Option<&str>,
    ) -> DriverResult<(Service, KeyWithValue)> {
        let user_allow_register = user_allow_register
            .unwrap_or("false")
            .parse::<bool>()
            .unwrap();
        let service_create = ServiceCreate {
            is_enabled: true,
            name: name.to_owned(),
            url: url.to_owned(),
            user_allow_register,
            user_email_text: user_email_text.unwrap_or("").to_owned(),
            provider_local_url: provider_local_url.map(|x| x.to_owned()),
            provider_github_oauth2_url: provider_github_oauth2_url.map(|x| x.to_owned()),
            provider_microsoft_oauth2_url: provider_microsoft_oauth2_url.map(|x| x.to_owned()),
        };
        let service = driver.service_create(&service_create)?;
        let key_create = KeyCreate::service(true, name, service.id);
        let key = driver.key_create(&key_create)?;
        Ok((service, key))
    }
}
