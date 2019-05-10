use crate::{
    core,
    server::{
        route::auth::oauth2::{oauth2_redirect, CallbackQuery, UrlResponse},
        route_response_json, ConfigurationOauth2Provider, Data, Error, FromJsonValue,
    },
};
use actix_web::{
    http::{header, StatusCode},
    middleware::identity::Identity,
    web, HttpResponse, ResponseError,
};
use futures::{future, Future};
use oauth2::prelude::*;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
struct GithubUser {
    email: String,
}

pub fn request_handler(
    data: web::Data<Data>,
    id: Identity,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    web::block(move || request_inner(data.get_ref(), id))
        .map_err(Into::into)
        .then(route_response_json)
}

fn request_inner(data: &Data, id: Option<String>) -> Result<UrlResponse, Error> {
    core::key::authenticate_service(data.driver(), id)
        .map_err(Into::into)
        .and_then(|service| github_authorise(&data, &service).map_err(Into::into))
        .map(|url| UrlResponse { url })
}

pub fn callback_handler(
    data: web::Data<Data>,
    query: web::Query<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    CallbackQuery::from_value(query.into_inner())
        .and_then(|query| {
            web::block(move || {
                let (service_id, access_token) =
                    github_callback(data.get_ref(), &query.code, &query.state)?;
                Ok((data, service_id, access_token))
            })
            .map_err(Into::into)
        })
        .and_then(|(data, service_id, access_token)| {
            let email = github_api_user_email(data.get_ref(), &access_token);
            let service_id = future::ok(service_id);
            let data = future::ok(data);
            data.join3(service_id, email)
        })
        .and_then(|(data, service_id, email)| {
            web::block(move || {
                core::auth::oauth2_login(data.driver(), service_id, &email).map_err(Into::into)
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
    let client = github_client(data.configuration().oauth2_github())?;
    let (authorize_url, state) = client.authorize_url(CsrfToken::new_random);

    // Save the state and code verifier secrets as a CSRF key, value.
    core::csrf::create(data.driver(), service, &state.secret(), &state.secret())
        .map_err(Error::Core)?;

    Ok(authorize_url.to_string())
}

fn github_callback(data: &Data, code: &str, state: &str) -> Result<(i64, String), Error> {
    // Read the CSRF key using state value, rebuild code verifier from value.
    let csrf = core::csrf::read_by_key(data.driver(), &state).map_err(Error::Core)?;
    let csrf = csrf.ok_or_else(|| Error::Oauth2)?;

    // Exchange the code with a token.
    let client = github_client(data.configuration().oauth2_github())?;
    let code = AuthorizationCode::new(code.to_owned());
    let token = client.exchange_code(code).map_err(|_err| Error::Oauth2)?;

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
        .map_err(|_err| Error::Oauth2)
        .and_then(|res| match res.status() {
            StatusCode::OK => future::ok(res),
            _ => future::err(Error::Oauth2),
        })
        .and_then(|mut res| res.json::<GithubUser>().map_err(|_err| Error::Oauth2))
        .map(|res| res.email)
}

fn github_client(provider: Option<&ConfigurationOauth2Provider>) -> Result<BasicClient, Error> {
    let provider = provider.ok_or(Error::Oauth2)?;

    let github_client_id = ClientId::new(provider.client_id.to_owned());
    let github_client_secret = ClientSecret::new(provider.client_secret.to_owned());
    let auth_url = AuthUrl::new(Url::parse("https://github.com/login/oauth/authorize").unwrap());
    let token_url =
        TokenUrl::new(Url::parse("https://github.com/login/oauth/access_token").unwrap());

    Ok(BasicClient::new(
        github_client_id,
        Some(github_client_secret),
        auth_url,
        Some(token_url),
    )
    .add_scope(Scope::new("user:email".to_string()))
    .set_redirect_url(RedirectUrl::new(
        Url::parse(&provider.redirect_url).unwrap(),
    )))
}
