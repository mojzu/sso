use crate::core;
use crate::server::{
    auth::oauth2::{oauth2_redirect, CallbackQuery, UrlResponse},
    route_response_json, ConfigurationOauth2Provider, Data, Error, ValidateFromValue,
};
use actix_web::{
    http::{header, StatusCode},
    middleware::identity::Identity,
    web, HttpResponse, ResponseError,
};
use futures::{future, Future};
use oauth2::prelude::*;
use oauth2::{
    basic::BasicClient, AuthType, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeVerifierS256, RedirectUrl, ResponseType, Scope, TokenResponse, TokenUrl,
};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
struct MicrosoftUser {
    mail: String,
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
    core::service::authenticate(data.driver(), id)
        .map_err(Into::into)
        .and_then(|service| microsoft_authorise(&data, &service).map_err(Into::into))
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
                    microsoft_callback(data.get_ref(), &query.code, &query.state)?;
                Ok((data, service_id, access_token))
            })
            .map_err(Into::into)
        })
        .and_then(|(data, service_id, access_token)| {
            let email = microsoft_api_user_email(data.get_ref(), &access_token);
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

fn microsoft_authorise(data: &Data, service: &core::Service) -> Result<String, Error> {
    // Microsoft Graph supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
    let code_verifier = PkceCodeVerifierS256::new_random();

    // Generate the authorisation URL to redirect.
    let client = microsoft_client(data.configuration().oauth2_microsoft())?;
    let (authorize_url, state) = client.authorize_url_extension(
        &ResponseType::new("code".to_string()),
        CsrfToken::new_random,
        // Send the PKCE code challenge in the authorisation request
        &code_verifier.authorize_url_params(),
    );

    // Save the state and code verifier secrets as a CSRF key, value.
    core::csrf::create(
        data.driver(),
        service,
        &state.secret(),
        &code_verifier.secret(),
    )
    .map_err(Error::Core)?;

    Ok(authorize_url.to_string())
}

fn microsoft_callback(data: &Data, code: &str, state: &str) -> Result<(i64, String), Error> {
    // Read the CSRF key using state value, rebuild code verifier from value.
    let csrf = core::csrf::read_by_key(data.driver(), &state).map_err(Error::Core)?;
    let csrf = csrf.ok_or_else(|| Error::Oauth2)?;

    // Send the PKCE code verifier in the token request
    let params: Vec<(&str, &str)> = vec![("code_verifier", &csrf.value)];

    // Exchange the code with a token.
    let client = microsoft_client(data.configuration().oauth2_microsoft())?;
    let code = AuthorizationCode::new(code.to_owned());
    let token = client
        .exchange_code_extension(code, &params)
        .map_err(|_err| Error::Oauth2)?;

    // Return access token value.
    Ok((csrf.service_id, token.access_token().secret().to_owned()))
}

fn microsoft_api_user_email(
    data: &Data,
    access_token: &str,
) -> impl Future<Item = String, Error = Error> {
    let client = actix_web::client::Client::new();
    let authorisation_header = format!("Bearer {}", access_token);
    client
        .get("https://graph.microsoft.com/v1.0/me")
        .header(header::AUTHORIZATION, authorisation_header)
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::USER_AGENT, data.configuration().user_agent())
        .send()
        .map_err(|_err| Error::Oauth2)
        .and_then(|res| match res.status() {
            StatusCode::OK => future::ok(res),
            _ => future::err(Error::Oauth2),
        })
        .and_then(|mut res| res.json::<MicrosoftUser>().map_err(|_err| Error::Oauth2))
        .map(|res| res.mail)
}

fn microsoft_client(provider: Option<&ConfigurationOauth2Provider>) -> Result<BasicClient, Error> {
    let provider = provider.ok_or(Error::Oauth2)?;

    let graph_client_id = ClientId::new(provider.client_id.to_owned());
    let graph_client_secret = ClientSecret::new(provider.client_secret.to_owned());
    let auth_url = AuthUrl::new(
        Url::parse("https://login.microsoftonline.com/common/oauth2/v2.0/authorize").unwrap(),
    );
    let token_url = TokenUrl::new(
        Url::parse("https://login.microsoftonline.com/common/oauth2/v2.0/token").unwrap(),
    );

    Ok(BasicClient::new(
        graph_client_id,
        Some(graph_client_secret),
        auth_url,
        Some(token_url),
    )
    .add_scope(Scope::new(
        "https://graph.microsoft.com/User.Read".to_string(),
    ))
    .set_auth_type(AuthType::RequestBody)
    .set_redirect_url(RedirectUrl::new(
        Url::parse(&provider.redirect_url).unwrap(),
    )))
}
