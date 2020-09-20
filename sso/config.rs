//! # Configuration
use crate::internal::*;

/// Log Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigLog {
    /// Pretty logs enable flag
    pub pretty: bool,
}

/// HTTP Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigHttp {
    /// Public server configuration
    pub public: ConfigHttpIf,
    /// Private server configuration
    pub private: ConfigHttpIf,
    /// Cookie configuration
    pub cookie: ConfigHttpCookie,
}

/// HTTP Interface Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigHttpIf {
    /// Bind address
    pub bind: String,
}

/// HTTP Cookie Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigHttpCookie {
    /// Master cookie encryption/signing key
    pub key: String,
    /// Cookie name
    pub name: String,
    /// Cookie domain
    pub domain: String,
    /// Cookie path
    pub path: String,
    /// Cookie secure
    pub secure: bool,
    /// Cookie same site (strict or lax)
    pub same_site: String,
    /// Cookie max age
    pub max_age: i64,
}

/// OAuth2 Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2 {
    /// Public domain
    pub domain: Url,
    /// Clients configuration
    pub clients: HashMap<Uuid, ConfigOauth2Client>,
    /// Users configuration
    pub users: HashMap<Uuid, ConfigOauth2User>,
    /// Providers configuration
    #[serde(default)]
    pub providers: ConfigOauth2Providers,
}

/// OAuth2 Client Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2Client {
    /// Client name
    pub name: String,
    /// Client URI
    pub uri: Url,
    /// Client secret
    pub secret: String,
    /// Client redirect URI
    pub redirect_uri: Url,
    /// Enable flag
    #[serde(default = "default_as_true")]
    pub enable: bool,
    /// Scope assigned to client
    #[serde(default)]
    pub scope: Vec<String>,
    /// Scope available to users
    #[serde(default)]
    pub user_scope: Vec<String>,
    /// Allow user registration for client
    #[serde(default)]
    pub register_enable: bool,
    /// Scope given to user on registration with client
    #[serde(default)]
    pub register_scope: Vec<String>,
    /// Token TTL configuration
    #[serde(default)]
    pub ttl: ConfigOauth2ClientTtl,
    /// Template configuration
    #[serde(default)]
    pub templates: ConfigOauth2ClientTemplates,
}

/// OAuth2 Client TTL Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2ClientTtl {
    /// Number of seconds a CSRF token is valid for
    #[serde(default = "default_as_3600")]
    pub csrf_s: i64,
    /// Number of seconds a code is valid for
    #[serde(default = "default_as_3600")]
    pub code_s: i64,
    /// Number of seconds an OAuth2 code is valid for
    #[serde(default = "default_as_3600")]
    pub oauth2_code_s: i64,
    /// Number of seconds an access token is valid for
    #[serde(default = "default_as_3600")]
    pub token_access_s: i64,
    /// Number of seconds after access token expiry that refresh token is valid for
    #[serde(default = "default_as_86400")]
    pub token_refresh_s: i64,
}

/// OAuth2 Client Templates Configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2ClientTemplates {
    /// HTML template
    #[serde(default)]
    pub html: ConfigOauth2ClientTemplate,
    /// Mail text template
    #[serde(default)]
    pub mail_text: ConfigOauth2ClientTemplate,
}

/// OAuth2 Client Template Configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2ClientTemplate {
    /// Template file
    pub file: Option<String>,
    /// Template string
    #[serde(default)]
    pub content: String,
}

/// OAuth2 User Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2User {
    /// User name
    pub name: String,
    /// User email address
    pub email: String,
    /// User password hash (optional)
    pub password: Option<String>,
    /// User locale
    #[serde(default)]
    pub locale: String,
    /// User timezone
    #[serde(default)]
    pub timezone: String,
    /// User enable flag
    #[serde(default = "default_as_true")]
    pub enable: bool,
    /// User access
    pub access: HashMap<Uuid, ConfigOauth2UserAccess>,
}

/// OAuth2 User Access Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2UserAccess {
    /// Access enable flag
    #[serde(default = "default_as_true")]
    pub enable: bool,
    /// Access scope
    #[serde(default)]
    pub scope: Vec<String>,
}

/// OAuth2 Providers Configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2Providers {
    /// SSO provider
    pub sso: Option<ConfigOauth2Provider>,
    /// Microsoft provider
    pub microsoft: Option<ConfigOauth2Provider>,
}

/// OAuth2 Provider Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2Provider {
    /// Client ID
    pub client_id: String,
    /// Client secret
    pub client_secret: String,
    /// Authorization URI
    pub authorize_uri: Url,
    /// Token URI
    pub token_uri: Url,
    /// Introspection URI
    pub introspect_uri: Option<Url>,
    /// OIDC userinfo URI
    pub oidc_userinfo_uri: Option<Url>,
}

/// Configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Log configuration
    pub log: ConfigLog,
    /// HTTP server configuration
    pub http: ConfigHttp,
    /// OAuth2 configuration
    pub oauth2: ConfigOauth2,
    /// Postgres configuration
    pub postgres: deadpool_postgres::Config,
    /// Mailto configuration
    #[serde(default)]
    pub mailto: mailto::Config,
    /// Metrics configuration
    #[serde(default)]
    pub metrics: metrics::Config,
}

/// Parse configuration from environment variables
pub fn from_env(file_name: &str) -> Result<Config> {
    Config::from_env(file_name)
}

impl Config {
    /// Parse configuration from environment variables
    pub fn from_env(file_name: &str) -> Result<Self> {
        let mut cfg = config::Config::new();

        cfg.merge(config::File::from_str(
            &format!(
                r#"

                [log]
                pretty = false

                [http.public]
                bind = "0.0.0.0:7042"

                [http.private]
                bind = "0.0.0.0:7043"

                [http.cookie]
                key = ""
                name = "sso.id"
                domain = ""
                path = "/"
                secure = true
                same_site = "strict"
                max_age = 604800

                [oauth2]
                domain = ""
                path = ""

                [postgres]
                application_name = "{}"

                "#,
                util::NAME
            ),
            config::FileFormat::Toml,
        ))?;
        cfg.merge(config::File::with_name(file_name))?;

        cfg.merge(config::Environment::with_prefix(util::NAME).separator("_"))?;

        cfg.try_into().map_err(Error::ConfigError)
    }

    /// Load configuration template files
    pub async fn load_templates(mut self) -> Result<Self> {
        let mut clients: HashMap<Uuid, ConfigOauth2Client> = HashMap::new();

        for (id, client) in self.oauth2.clients {
            let mut client = client.clone();
            if let Some(html) = &client.templates.html.file {
                client.templates.html.content = Self::load_template_file(html).await?;
            }
            if let Some(mail_text) = &client.templates.mail_text.file {
                client.templates.mail_text.content = Self::load_template_file(mail_text).await?;
            }
            clients.insert(id, client);
        }

        self.oauth2.clients = clients;
        Ok(self)
    }

    async fn load_template_file(file: &str) -> Result<String> {
        use tokio::fs;
        let contents = fs::read_to_string(file).await?;
        Ok(contents)
    }
}

impl ConfigHttpCookie {
    /// Returns bytes of key string, if empty random bytes are generated
    ///
    /// Rand is not a crypto library and the random keys it generates must
    /// not be used in production. This is a developer ease of use feature,
    /// in production the cookie key must be explicitly configured.
    pub fn key_bytes(&self) -> Vec<u8> {
        use rand::Rng;

        if self.key.is_empty() {
            rand::thread_rng().gen::<[u8; 32]>().to_vec()
        } else {
            self.key.clone().into_bytes()
        }
    }
}

impl Default for ConfigOauth2ClientTtl {
    fn default() -> Self {
        Self {
            csrf_s: 3600,
            code_s: 3600,
            oauth2_code_s: 3600,
            token_access_s: 3600,
            token_refresh_s: 86400,
        }
    }
}
