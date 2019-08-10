use crate::client::{Client, Get};
use crate::core;
use crate::core::audit::AuditBuilder;
use crate::core::AuditMeta;
use crate::server::api::{path, AuthOauth2CallbackQuery, AuthOauth2UrlResponse};
use crate::server::route::auth::provider::oauth2_redirect;
use crate::server::route::{request_audit_meta, route_response_json};
use crate::server::{Data, Error, FromJsonValue, Oauth2Error, ServerOptionsProviderOauth2};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use futures::{future, Future};
use oauth2::reqwest::http_client;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use serde_json::Value;
use url::Url;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::GITHUB).service(
        web::resource(path::OAUTH2)
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
) -> Result<AuthOauth2UrlResponse, Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .map_err(Into::into)
        .and_then(|(service, mut audit)| {
            github_authorise(&data, &service, &mut audit).map_err(Into::into)
        })
        .map(|url| AuthOauth2UrlResponse { url })
}

fn oauth2_callback_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    query: web::Query<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let audit_meta = request_audit_meta(&req);
    let query = AuthOauth2CallbackQuery::from_value(query.into_inner());

    audit_meta
        .join(query)
        .and_then(|(audit_meta, query)| {
            web::block(move || {
                let mut audit = AuditBuilder::new(audit_meta);
                let (service_id, access_token) =
                    github_callback(data.get_ref(), &mut audit, &query.code, &query.state)?;
                Ok((data, audit, service_id, access_token))
            })
            .map_err(Into::into)
        })
        .and_then(|(data, audit, service_id, access_token)| {
            let email = github_api_user_email(data.get_ref(), &access_token);
            let args = future::ok((data, audit, service_id));
            email.join(args)
        })
        .and_then(|(email, (data, mut audit, service_id))| {
            web::block(move || {
                core::auth::oauth2_login(
                    data.driver(),
                    &service_id,
                    &mut audit,
                    &email,
                    data.options().access_token_expires(),
                    data.options().refresh_token_expires(),
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

fn github_authorise(
    data: &Data,
    service: &core::Service,
    _audit: &mut AuditBuilder,
) -> Result<String, Error> {
    // Generate the authorization URL to which we'll redirect the user.
    let client = github_client(data.options().provider_github_oauth2())?;
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
        data.options().access_token_expires(),
    )
    .map_err(Error::Core)?;

    Ok(authorise_url.to_string())
}

fn github_callback(
    data: &Data,
    _audit: &mut AuditBuilder,
    code: &str,
    state: &str,
) -> Result<(String, String), Error> {
    // Read the CSRF key using state value, rebuild code verifier from value.
    let csrf = core::csrf::read_by_key(data.driver(), &state).map_err(Error::Core)?;
    let csrf = csrf.ok_or_else(|| Error::Oauth2(Oauth2Error::Csrf))?;

    // Exchange the code with a token.
    let client = github_client(data.options().provider_github_oauth2())?;
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
    let authorisation = format!("token {}", access_token);
    data.client()
        .send(Get::json("https://api.github.com", "/user").authorisation(authorisation))
        .map_err(Error::ActixMailbox)
        .and_then(|res| Client::result_json::<GithubUser>(res).map_err(Error::Client))
        .map(|res| res.email)
}

fn github_client(provider: Option<&ServerOptionsProviderOauth2>) -> Result<BasicClient, Error> {
    let provider = provider.ok_or_else(|| {
        // Warn OAuth2 is disabled, return bad request error so internal server error
        // is not returned to the client.
        let err = Error::Oauth2(Oauth2Error::Disabled);
        warn!("{}", err);
        Error::BadRequest
    })?;

    let github_client_id = ClientId::new(provider.client_id.to_owned());
    let github_client_secret = ClientSecret::new(provider.client_secret.to_owned());

    let auth_url =
        Url::parse("https://github.com/login/oauth/authorize").map_err(Error::UrlParse)?;
    let auth_url = AuthUrl::new(auth_url);
    let token_url =
        Url::parse("https://github.com/login/oauth/access_token").map_err(Error::UrlParse)?;
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
