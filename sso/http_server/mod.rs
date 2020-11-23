//! # HTTP Server
#[macro_use]
mod internal;
mod api;
mod error;
mod route_audit;
mod route_auth;
mod route_client;
mod route_client_access;
mod route_csrf;
mod route_oauth2;
mod route_script;
mod route_user;
mod route_user_access;
mod route_user_api_key;
mod route_well_known;
mod template;

pub use crate::http_server::error::*;

use crate::internal::*;
use ::oauth2::basic::BasicClient;

/// HTTP Server
#[derive(Clone)]
pub struct HttpServer {
    config: Arc<Config>,
    postgres: Postgres,
    client: reqwest::Client,
    handlebars: Arc<handlebars::Handlebars<'static>>,
    mailto: mailto::Mailto,
    metrics: metrics::Metrics,
    oauth2_providers: ServerOauth2Providers,
    opentelemetry: Arc<ServerOpentelemetry>,
}

/// Create HTTP server from configuration
pub async fn from_config(config: Config) -> Result<HttpServer> {
    HttpServer::from_config(config).await
}

impl HttpServer {
    /// Returns configuration
    pub fn config(&self) -> &Config {
        self.config.as_ref()
    }

    /// Returns mailto
    pub fn mailto(&self) -> &mailto::Mailto {
        &self.mailto
    }

    /// Returns metrics
    pub fn metrics(&self) -> &metrics::Metrics {
        &self.metrics
    }

    /// Create HTTP server from configuration
    pub async fn from_config(config: Config) -> Result<Self> {
        let postgres = Postgres::from_config(&config).await?;

        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .user_agent(util::USER_AGENT)
            .build()?;

        let sso =
            Self::oauth2_provider_client_sso(&config.oauth2.domain, &config.oauth2.providers.sso)?;
        let microsoft = Self::oauth2_provider_client_microsoft(
            &config.oauth2.domain,
            &config.oauth2.providers.microsoft,
        )?;

        let metrics = metrics::from_config(config.metrics.clone())?;
        let opentelemetry = Arc::new(ServerOpentelemetry {
            http_req_count: metrics
                .meter()
                .u64_counter("http_req_count")
                .with_description("Total number of HTTP requests.")
                .init()
                .bind(&[]),
            http_req_histogram: metrics
                .meter()
                .f64_value_recorder("http_req_histogram")
                .with_description("HTTP request latencies in seconds.")
                .init()
                .bind(&[]),
            validation_err_count: metrics
                .meter()
                .u64_counter("validation_err_count")
                .with_description("Total number of validation errors.")
                .init()
                .bind(&[]),
            api_ok_count: metrics
                .meter()
                .u64_counter("api_ok_count")
                .with_description("Total number of api successes.")
                .init()
                .bind(&[]),
            api_err_count: metrics
                .meter()
                .u64_counter("api_err_count")
                .with_description("Total number of api errors.")
                .init()
                .bind(&[]),
        });

        Ok(Self {
            config: Arc::new(config.clone()),
            postgres,
            client,
            handlebars: Arc::new(handlebars::Handlebars::new()),
            mailto: mailto::from_config(&metrics, config.mailto.clone()).await?,
            metrics,
            oauth2_providers: ServerOauth2Providers { sso, microsoft },
            opentelemetry,
        })
    }

    /// Create public service server
    pub fn public_service(&self) -> Result<actix_server::Server> {
        let server = self.clone();
        let bind = &self.config.http.public.bind;
        let cookie_key = self.config.http.cookie.key_bytes();
        let cookie_name = self.config.http.cookie.name.clone();
        let cookie_domain = self.config.http.cookie.domain.clone();
        let cookie_path = self.config.http.cookie.path.clone();
        let cookie_secure = self.config.http.cookie.secure;
        let cookie_same_site = Self::cookie_same_site(&self.config.http.cookie.same_site)
            .expect("failed to parse same_site");
        let cookie_max_age = self.config.http.cookie.max_age;

        Ok(actix_web::HttpServer::new(move || {
            let mut spec = paperclip::v2::models::DefaultApiRaw::default();
            spec.info = paperclip::v2::models::Info {
                version: util::API_VERSION.into(),
                title: format!("{} (Public)", util::API_TITLE),
                ..Default::default()
            };

            actix_web::App::new()
                .wrap_api_with_spec(spec)
                .wrap(actix_identity::IdentityService::new(
                    actix_identity::CookieIdentityPolicy::new(&cookie_key)
                        .name(&cookie_name)
                        .domain(&cookie_domain)
                        .path(&cookie_path)
                        .secure(cookie_secure)
                        .max_age(cookie_max_age)
                        .same_site(cookie_same_site),
                ))
                .data(server.clone())
                .with_json_spec_at("openapi.json")
                .service(api::ServerApi::public())
                .build()
        })
        .disable_signals()
        .bind(bind)?
        .run())
    }

    /// Create private service server
    pub fn private_service(&self) -> Result<actix_server::Server> {
        let server = self.clone();
        let bind = &self.config.http.private.bind;

        Ok(actix_web::HttpServer::new(move || {
            let mut spec = paperclip::v2::models::DefaultApiRaw::default();
            spec.info = paperclip::v2::models::Info {
                version: util::API_VERSION.into(),
                title: format!("{} (Private)", util::API_TITLE),
                ..Default::default()
            };

            actix_web::App::new()
                .wrap_api_with_spec(spec)
                .data(server.clone())
                .with_json_spec_at("openapi.json")
                .service(api::ServerApi::private())
                .build()
        })
        .disable_signals()
        .workers(1)
        .bind(bind)?
        .run())
    }

    /// Returns ok if server is ready to accept requests
    pub(crate) async fn readiness(&self) -> HttpResult<()> {
        if let Err(e) = self
            .postgres
            .readiness()
            .await
            .map_err(HttpError::InternalServerError)
        {
            error!("readiness postgres check failure: {}", e);
            return Err(e);
        }
        if let Err(e) = self
            .readiness_http()
            .await
            .map_err(HttpError::InternalServerError)
        {
            error!("readiness http check failure: {}", e);
            return Err(e);
        }
        Ok(())
    }

    /// Returns ok if client can make request to HTTP public ping endpoint
    async fn readiness_http(&self) -> Result<()> {
        let url = format!("http://{}/ping", self.config.http.public.bind);

        self.client.get(&url).send().await?.error_for_status()?;

        Ok(())
    }

    pub(crate) fn uri_oauth2_authorize(&self) -> Url {
        ServerUri::oauth2_authorize(&self.config.oauth2.domain)
    }

    pub(crate) fn uri_oauth2_token(&self) -> Url {
        ServerUri::oauth2_token(&self.config.oauth2.domain)
    }

    pub(crate) fn uri_oauth2_introspect(&self) -> Url {
        ServerUri::oauth2_introspect(&self.config.oauth2.domain)
    }

    pub(crate) fn uri_auth_password_reset(&self, code: &str) -> Url {
        ServerUri::auth_password_reset(&self.config.oauth2.domain, code)
    }

    pub(crate) fn uri_auth_password_update(&self, client: &Client, message: Option<&str>) -> Url {
        ServerUri::auth_password_update(
            &self.config.oauth2.domain,
            &client.client_id(),
            &client.redirect_uri().to_string(),
            message,
        )
    }

    pub(crate) fn uri_auth_register(&self, code: &str) -> Url {
        ServerUri::auth_register(&self.config.oauth2.domain, code)
    }

    pub(crate) fn uri_auth_delete(&self, code: &str) -> Url {
        ServerUri::auth_delete(&self.config.oauth2.domain, code)
    }
}

impl oauth2::AuthorizationServerIf for HttpServer {}

impl HttpServer {
    fn oauth2_provider_client_microsoft(
        domain: &Url,
        provider: &Option<ConfigOauth2Provider>,
    ) -> Result<Option<BasicClient>> {
        if let Some(provider) = provider {
            let client = BasicClient::new(
                ::oauth2::ClientId::new(provider.client_id.clone()),
                Some(::oauth2::ClientSecret::new(provider.client_secret.clone())),
                ::oauth2::AuthUrl::from_url(provider.authorize_uri.clone()),
                Some(::oauth2::TokenUrl::from_url(provider.token_uri.clone())),
            )
            .set_auth_type(::oauth2::AuthType::RequestBody)
            .set_redirect_url(::oauth2::RedirectUrl::from_url(ServerUri::oauth2_redirect(
                domain,
            )));
            Ok(Some(client))
        } else {
            Ok(None)
        }
    }

    fn oauth2_provider_client_sso(
        domain: &Url,
        provider: &Option<ConfigOauth2Provider>,
    ) -> Result<Option<BasicClient>> {
        if let Some(provider) = provider {
            let client = BasicClient::new(
                ::oauth2::ClientId::new(provider.client_id.clone()),
                Some(::oauth2::ClientSecret::new(provider.client_secret.clone())),
                ::oauth2::AuthUrl::from_url(provider.authorize_uri.clone()),
                Some(::oauth2::TokenUrl::from_url(provider.token_uri.clone())),
            )
            .set_redirect_url(::oauth2::RedirectUrl::from_url(
                ServerUri::oauth2_redirect(domain),
            ));
            Ok(Some(client))
        } else {
            Ok(None)
        }
    }

    fn cookie_same_site(value: &str) -> Result<actix_web::cookie::SameSite> {
        match value {
            "strict" => Ok(actix_web::cookie::SameSite::Strict),
            "lax" => Ok(actix_web::cookie::SameSite::Lax),
            _ => Err(Error::from("cookie.same_site invalid")),
        }
    }
}

/// Server Opentelemetry
#[derive(Debug)]
pub(crate) struct ServerOpentelemetry {
    pub http_req_count: BoundCounter<'static, u64>,
    pub http_req_histogram: BoundValueRecorder<'static, f64>,
    pub validation_err_count: BoundCounter<'static, u64>,
    pub api_ok_count: BoundCounter<'static, u64>,
    pub api_err_count: BoundCounter<'static, u64>,
}

/// Server OAuth2 Providers
#[derive(Clone)]
pub(crate) struct ServerOauth2Providers {
    pub sso: Option<BasicClient>,
    pub microsoft: Option<BasicClient>,
}

/// Server URI
pub(crate) struct ServerUri;

impl ServerUri {
    pub fn oauth2_authorize(domain: &Url) -> Url {
        let mut uri = domain.clone();
        uri.set_path("/v2/oauth2/authorize");
        uri
    }

    pub fn oauth2_token(domain: &Url) -> Url {
        let mut uri = domain.clone();
        uri.set_path("/v2/oauth2/token");
        uri
    }

    pub fn oauth2_introspect(domain: &Url) -> Url {
        let mut uri = domain.clone();
        uri.set_path("/v2/oauth2/introspect");
        uri
    }

    pub fn oauth2_redirect(domain: &Url) -> Url {
        let mut uri = domain.clone();
        uri.set_path("/v2/oauth2/redirect");
        uri
    }

    pub fn auth_password_reset(domain: &Url, code: &str) -> Url {
        let mut uri = domain.clone();
        uri.set_path("/v2/auth/password-reset");
        uri.query_pairs_mut().clear().append_pair("code", &code);
        uri
    }

    pub fn auth_password_update(
        domain: &Url,
        client_id: &str,
        redirect_uri: &str,
        message: Option<&str>,
    ) -> Url {
        let mut uri = domain.clone();
        uri.set_path("/v2/auth/password-update");
        uri.query_pairs_mut()
            .clear()
            .append_pair("client_id", &client_id)
            .append_pair("redirect_uri", &redirect_uri);
        if let Some(message) = message {
            uri.query_pairs_mut().append_pair("message", &message);
        }
        uri
    }

    pub fn auth_register(domain: &Url, code: &str) -> Url {
        let mut uri = domain.clone();
        uri.set_path("/v2/auth/register");
        uri.query_pairs_mut().clear().append_pair("code", &code);
        uri
    }

    pub fn auth_delete(domain: &Url, code: &str) -> Url {
        let mut uri = domain.clone();
        uri.set_path("/v2/auth/delete");
        uri.query_pairs_mut().clear().append_pair("code", &code);
        uri
    }
}

impl std::fmt::Debug for HttpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HttpServer {{ config, ... }}")
    }
}
