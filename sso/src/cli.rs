use crate::{Driver, DriverResult, KeyCreate, KeyWithValue, Service, ServiceCreate};

/// CLI functions.
#[derive(Debug)]
pub struct Cli;

impl Cli {
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
