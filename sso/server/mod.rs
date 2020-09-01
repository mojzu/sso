#[macro_use]
mod internal;
mod api;
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
mod template;

use crate::internal::*;
use ::oauth2::basic::BasicClient;

/// Server
#[derive(Clone)]
pub struct Server {
    pub config: Arc<Config>,
    pub postgres: Postgres,
    pub client: reqwest::Client,
    pub handlebars: Arc<handlebars::Handlebars<'static>>,
    pub mailto: Mailto,
    pub oauth2_providers: ServerOauth2Providers,
    pub opentelemetry: Arc<ServerOpentelemetry>,
}

/// Server Opentelemetry
#[derive(Debug)]
pub struct ServerOpentelemetry {
    pub exporter: opentelemetry_prometheus::PrometheusExporter,
    pub http_req_count: BoundCounter<'static, u64>,
    pub http_req_histogram: BoundValueRecorder<'static, f64>,
    pub validation_err_count: BoundCounter<'static, u64>,
    pub api_ok_count: BoundCounter<'static, u64>,
    pub api_err_count: BoundCounter<'static, u64>,
    pub mailto_ok_count: BoundCounter<'static, u64>,
    pub mailto_err_count: BoundCounter<'static, u64>,
}

/// Server OAuth2 Providers
#[derive(Clone)]
pub struct ServerOauth2Providers {
    pub sso: Option<BasicClient>,
    pub microsoft: Option<BasicClient>,
}

impl oauth2::AuthorizationServerIf for Server {}

impl Server {
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
}

impl Server {
    /// Create server from configuration
    pub async fn from_config(config: Config) -> Result<Self> {
        let postgres = Postgres::from_config(&config).await?;

        let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;

        let sso =
            Self::oauth2_provider_client_sso(&config.oauth2.domain, &config.oauth2.providers.sso)?;
        let microsoft = Self::oauth2_provider_client_microsoft(
            &config.oauth2.domain,
            &config.oauth2.providers.microsoft,
        )?;

        let exporter = opentelemetry_prometheus::exporter().init();
        let meter = opentelemetry::global::meter("sso");
        let opentelemetry = Arc::new(ServerOpentelemetry {
            exporter,
            http_req_count: meter
                .u64_counter("http_req_count")
                .with_description("Total number of HTTP requests.")
                .init()
                .bind(&[]),
            http_req_histogram: meter
                .f64_value_recorder("http_req_histogram")
                .with_description("HTTP request latencies in seconds.")
                .init()
                .bind(&[]),
            validation_err_count: meter
                .u64_counter("validation_err_count")
                .with_description("Total number of validation errors.")
                .init()
                .bind(&[]),
            api_ok_count: meter
                .u64_counter("api_ok_count")
                .with_description("Total number of api successes.")
                .init()
                .bind(&[]),
            api_err_count: meter
                .u64_counter("api_err_count")
                .with_description("Total number of api errors.")
                .init()
                .bind(&[]),
            mailto_ok_count: meter
                .u64_counter("mailto_ok_count")
                .with_description("Total number of mailto send successes.")
                .init()
                .bind(&[]),
            mailto_err_count: meter
                .u64_counter("mailto_err_count")
                .with_description("Total number of mailto send errors.")
                .init()
                .bind(&[]),
        });

        Ok(Self {
            config: Arc::new(config.clone()),
            postgres,
            client,
            handlebars: Arc::new(handlebars::Handlebars::new()),
            mailto: Mailto::from_config(config.mailto.clone(), opentelemetry.clone()).await,
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
        let cookie_max_age = self.config.http.cookie.max_age;

        Ok(actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .wrap_api()
                .wrap(actix_identity::IdentityService::new(
                    actix_identity::CookieIdentityPolicy::new(&cookie_key)
                        .name(&cookie_name)
                        .domain(&cookie_domain)
                        .path(&cookie_path)
                        .secure(cookie_secure)
                        .max_age(cookie_max_age)
                        .same_site(actix_web::cookie::SameSite::Strict),
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
            actix_web::App::new()
                .wrap_api()
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

    /// Returns Prometheus exposition text
    pub(crate) fn metrics(&self) -> actix_web::HttpResponse {
        use prometheus::{Encoder, TextEncoder};

        let mut buffer = vec![];
        let encoder = TextEncoder::new();

        let mut metric_families = prometheus::gather();
        let mut ot_metric_families = self.opentelemetry.exporter.registry().gather();
        metric_families.append(&mut ot_metric_families);
        encoder.encode(&metric_families, &mut buffer).unwrap();

        actix_web::HttpResponse::build(http::StatusCode::OK)
            .content_type(encoder.format_type())
            .body(buffer)
    }

    pub fn uri_oauth2_authorize(&self) -> Url {
        ServerUri::oauth2_authorize(&self.config.oauth2.domain)
    }

    pub fn uri_oauth2_token(&self) -> Url {
        ServerUri::oauth2_token(&self.config.oauth2.domain)
    }

    pub fn uri_oauth2_introspect(&self) -> Url {
        ServerUri::oauth2_introspect(&self.config.oauth2.domain)
    }

    pub fn uri_auth_password_reset(&self, code: &str) -> Url {
        ServerUri::auth_password_reset(&self.config.oauth2.domain, code)
    }

    pub fn uri_auth_password_update(&self, client: &Client, message: Option<&str>) -> Url {
        ServerUri::auth_password_update(
            &self.config.oauth2.domain,
            &client.client_id(),
            &client.redirect_uri().to_string(),
            message,
        )
    }

    pub fn uri_auth_register(&self, code: &str) -> Url {
        ServerUri::auth_register(&self.config.oauth2.domain, code)
    }

    pub fn uri_auth_delete(&self, code: &str) -> Url {
        ServerUri::auth_delete(&self.config.oauth2.domain, code)
    }
}

/// Server URI
pub struct ServerUri;

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
