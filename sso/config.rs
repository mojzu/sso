use crate::internal::*;

/// HTTP Interface Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigHttpIf {
    pub bind: String,
}

/// HTTP Cookie Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigHttpCookie {
    pub key: String,
    pub name: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub max_age: i64,
}

/// HTTP Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigHttp {
    pub public: ConfigHttpIf,
    pub private: ConfigHttpIf,
    pub cookie: ConfigHttpCookie,
}

/// OAuth2 Client Template Configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2ClientTemplate {
    pub file: Option<String>,
    #[serde(default)]
    pub content: String,
}

/// OAuth2 Client Templates Configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2ClientTemplates {
    #[serde(default)]
    pub html: ConfigOauth2ClientTemplate,
    #[serde(default)]
    pub mail_text: ConfigOauth2ClientTemplate,
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

/// OAuth2 Client Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2Client {
    pub name: String,
    pub uri: Url,
    pub secret: String,
    pub redirect_uri: Url,
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
    #[serde(default)]
    pub ttl: ConfigOauth2ClientTtl,
    #[serde(default)]
    pub templates: ConfigOauth2ClientTemplates,
}

/// OAuth2 User Access Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2UserAccess {
    #[serde(default = "default_as_true")]
    pub enable: bool,
    #[serde(default)]
    pub scope: Vec<String>,
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
    #[serde(default)]
    pub locale: String,
    #[serde(default)]
    pub timezone: String,
    #[serde(default = "default_as_true")]
    pub enable: bool,
    pub access: HashMap<Uuid, ConfigOauth2UserAccess>,
}

/// OAuth2 Provider Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2Provider {
    pub client_id: String,
    pub client_secret: String,
    pub authorize_uri: Url,
    pub token_uri: Url,
    pub introspect_uri: Option<Url>,
    pub oidc_userinfo_uri: Option<Url>,
}

/// OAuth2 Providers Configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2Providers {
    pub sso: Option<ConfigOauth2Provider>,
    pub microsoft: Option<ConfigOauth2Provider>,
}

/// OAuth2 Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOauth2 {
    pub domain: Url,
    pub path: String,
    pub clients: HashMap<Uuid, ConfigOauth2Client>,
    pub users: HashMap<Uuid, ConfigOauth2User>,
    #[serde(default)]
    pub providers: ConfigOauth2Providers,
}

/// Log Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigLog {
    pub pretty: bool,
}

/// Stdout Mailto Configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigMailtoStdout {
    pub enable: bool,
}

/// File Mailto Configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigMailtoFile {
    pub file: Option<String>,
}

/// SMTP Login Mailto Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMailtoSmtpLogin {
    pub user: String,
    pub password: String,
}

/// SMTP Mailto Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMailtoSmtp {
    pub host: String,
    pub port: u16,
    pub from: String,
    pub login: Option<ConfigMailtoSmtpLogin>,
}

/// Mailto Configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigMailto {
    #[serde(default)]
    pub stdout: ConfigMailtoStdout,
    #[serde(default)]
    pub file: ConfigMailtoFile,
    pub smtp: Option<ConfigMailtoSmtp>,
}

/// Configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub log: ConfigLog,
    pub http: ConfigHttp,
    pub oauth2: ConfigOauth2,
    pub postgres: deadpool_postgres::Config,
    pub mailto: ConfigMailto,
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
                max_age = 604800

                [oauth2]
                domain = ""
                path = ""

                [postgres]
                application_name = "{}"

                "#,
                NAME
            ),
            config::FileFormat::Toml,
        ))?;
        cfg.merge(config::File::with_name(file_name))?;

        cfg.merge(config::Environment::with_prefix(NAME).separator("_"))?;

        cfg.try_into().map_err(Error::ConfigError)
    }

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
