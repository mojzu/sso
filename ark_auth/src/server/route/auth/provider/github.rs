use crate::core;
use crate::core::audit::{AuditBuilder, AuditMeta};
use crate::server::route::auth::provider::{
    oauth2_redirect, Oauth2CallbackQuery, Oauth2UrlResponse,
};
use crate::server::route::{request_audit_meta, route_response_json};
use crate::server::{ConfigurationProviderOauth2, Data, Error, FromJsonValue, Oauth2Error};
use actix_identity::Identity;
use actix_web::http::{header, StatusCode};
use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use futures::{future, Future};
use oauth2::curl::http_client;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use serde_json::Value;
use url::Url;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope("/github").service(
        web::resource("/oauth2")
            .route(web::post().to_async(oauth2_request_handler))
            .route(web::get().to_async(oauth2_callback_handler)),
    )
}

#[derive(Debug, Serialize, Deserialize)]
struct GithubUser {
    email: String,
}

fn oauth2_request_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    request_audit_meta(&req)
        .and_then(|audit_meta| {
            web::block(move || request_inner(data.get_ref(), audit_meta, id)).map_err(Into::into)
        })
        .then(route_response_json)
}

fn request_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
) -> Result<Oauth2UrlResponse, Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .map_err(Into::into)
        .and_then(|(service, _)| github_authorise(&data, &service).map_err(Into::into))
        .map(|url| Oauth2UrlResponse { url })
}

fn oauth2_callback_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    query: web::Query<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let audit_meta = request_audit_meta(&req);
    let query = Oauth2CallbackQuery::from_value(query.into_inner());

    audit_meta
        .join(query)
        .and_then(|(audit_meta, query)| {
            web::block(move || {
                let (service_id, access_token) =
                    github_callback(data.get_ref(), &query.code, &query.state)?;
                Ok((data, audit_meta, service_id, access_token))
            })
            .map_err(Into::into)
        })
        .and_then(|(data, audit_meta, service_id, access_token)| {
            let email = github_api_user_email(data.get_ref(), &access_token);
            let args = future::ok((data, audit_meta, service_id));
            email.join(args)
        })
        .and_then(|(email, (data, audit_meta, service_id))| {
            web::block(move || {
                let audit = AuditBuilder::new(audit_meta);
                core::auth::oauth2_login(
                    data.driver(),
                    &service_id,
                    audit,
                    &email,
                    data.configuration().core_access_token_expires(),
                    data.configuration().core_refresh_token_expires(),
                )
                .map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(|res| match res {
            Ok((service, token)) => future::ok(oauth2_redirect(service, token)),
            Err(err) => future::ok(err.error_response()),
        })
}

fn github_authorise(data: &Data, service: &core::Service) -> Result<String, Error> {
    // Generate the authorization URL to which we'll redirect the user.
    let client = github_client(data.configuration().provider_github_oauth2())?;
    let (authorise_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    // Save the state and code verifier secrets as a CSRF key, value.
    core::csrf::create(
        data.driver(),
        service,
        &csrf_state.secret(),
        &csrf_state.secret(),
        data.configuration().core_access_token_expires(),
    )
    .map_err(Error::Core)?;

    Ok(authorise_url.to_string())
}

fn github_callback(data: &Data, code: &str, state: &str) -> Result<(String, String), Error> {
    // Read the CSRF key using state value, rebuild code verifier from value.
    let csrf = core::csrf::read_by_key(data.driver(), &state).map_err(Error::Core)?;
    let csrf = csrf.ok_or_else(|| Error::Oauth2(Oauth2Error::Csrf))?;

    // Exchange the code with a token.
    // TODO(refactor): Use async client.
    let client = github_client(data.configuration().provider_github_oauth2())?;
    let code = AuthorizationCode::new(code.to_owned());
    let token = client
        .exchange_code(code)
        .request(http_client)
        .map_err(|err| Error::Oauth2(Oauth2Error::Oauth2Request(err.into())))?;

    // Return access token value.
    Ok((csrf.service_id, token.access_token().secret().to_owned()))
}

fn github_api_user_email(
    data: &Data,
    access_token: &str,
) -> impl Future<Item = String, Error = Error> {
    let client = actix_web::client::Client::new();
    let authorisation_header = format!("token {}", access_token);
    client
        .get("https://api.github.com/user")
        .header(header::AUTHORIZATION, authorisation_header)
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::USER_AGENT, data.configuration().user_agent())
        .send()
        .map_err(|_err| Error::Oauth2(Oauth2Error::ActixClientSendRequest))
        .and_then(|res| {
            let status = res.status();
            match status {
                StatusCode::OK => future::ok(res),
                _ => future::err(Error::Oauth2(Oauth2Error::StatusCode(status))),
            }
        })
        .and_then(|mut res| {
            res.json::<GithubUser>()
                .map_err(|_err| Error::Oauth2(Oauth2Error::ActixPayload))
        })
        .map(|res| res.email)
}

fn github_client(provider: Option<&ConfigurationProviderOauth2>) -> Result<BasicClient, Error> {
    let provider = provider.ok_or(Error::Oauth2(Oauth2Error::Disabled))?;

    let github_client_id = ClientId::new(provider.client_id.to_owned());
    let github_client_secret = ClientSecret::new(provider.client_secret.to_owned());

    // Safe to unwrap here, known valid URLs.
    let auth_url = Url::parse("https://github.com/login/oauth/authorize").unwrap();
    let auth_url = AuthUrl::new(auth_url);
    let token_url = Url::parse("https://github.com/login/oauth/access_token").unwrap();
    let token_url = TokenUrl::new(token_url);

    let redirect_url = Url::parse(&provider.redirect_url).map_err(Error::UrlParse)?;
    Ok(BasicClient::new(
        github_client_id,
        Some(github_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_url(RedirectUrl::new(redirect_url)))
}
