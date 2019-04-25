use crate::api::auth::oauth2::{oauth2_login, oauth2_redirect, CallbackQuery, UrlResponse};
use crate::api::{authenticate, ApiConfigOauth2Provider, ApiData, ApiError};
use crate::models::AuthService;
use actix_http::http::header::ContentType;
use actix_web::http::{header, StatusCode};
use actix_web::middleware::identity::Identity;
use actix_web::{web, Error, HttpResponse};
use futures::{future, Future};
use oauth2::basic::BasicClient;
use oauth2::prelude::*;
use oauth2::{
    AuthType, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeVerifierS256,
    RedirectUrl, ResponseType, Scope, TokenResponse, TokenUrl,
};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct MicrosoftUser {
    pub mail: String,
}

pub fn v1(
    data: web::Data<ApiData>,
    id: Identity,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    web::block(move || authenticate(&data, id).and_then(|s| microsoft_authorise(&data, s)))
        .map_err(Into::into)
        .map(|x| HttpResponse::build(StatusCode::OK).json(UrlResponse { url: x }))
}

pub fn v1_callback(
    data: web::Data<ApiData>,
    query: web::Query<CallbackQuery>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || microsoft_callback(data, &query.code, &query.state))
        .map_err(Into::into)
        .and_then(move |(data, access_token, service_id)| {
            microsoft_api_user_email(data, &access_token, service_id)
        })
        .and_then(|(data, email, service_id)| {
            web::block(move || oauth2_login(&data, &email, service_id)).map_err(Into::into)
        })
        .map_err(Into::into)
        .map(|(token, service)| oauth2_redirect(token, service))
}

fn microsoft_authorise(
    data: &web::Data<ApiData>,
    service: AuthService,
) -> Result<String, ApiError> {
    // Microsoft Graph supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
    let code_verifier = PkceCodeVerifierS256::new_random();

    // Generate the authorisation URL to redirect.
    let client = microsoft_client(data.oauth2_microsoft())?;
    let (authorize_url, state) = client.authorize_url_extension(
        &ResponseType::new("code".to_string()),
        CsrfToken::new_random,
        // Send the PKCE code challenge in the authorisation request
        &code_verifier.authorize_url_params(),
    );

    // Save the state and code verifier secrets as a CSRF key, value.
    data.db
        .csrf_create(&state.secret(), &code_verifier.secret(), service.service_id)
        .map_err(ApiError::Db)?;

    Ok(authorize_url.to_string())
}

fn microsoft_callback(
    data: web::Data<ApiData>,
    code: &str,
    state: &str,
) -> Result<(web::Data<ApiData>, String, i64), ApiError> {
    // Read the CSRF key using state value, rebuild code verifier from value.
    let csrf = data.db.csrf_read_by_key(&state).map_err(ApiError::Db)?;

    // Send the PKCE code verifier in the token request
    let params: Vec<(&str, &str)> = vec![("code_verifier", &csrf.csrf_value)];

    // Exchange the code with a token.
    let client = microsoft_client(data.oauth2_microsoft())?;
    let code = AuthorizationCode::new(code.to_owned());
    let token = client
        .exchange_code_extension(code, &params)
        .map_err(|_e| ApiError::Unwrap("failed to exchange code"))?;

    // Return access token value.
    Ok((
        data,
        token.access_token().secret().to_owned(),
        csrf.service_id,
    ))
}

fn microsoft_api_user_email(
    data: web::Data<ApiData>,
    access_token: &str,
    service_id: i64,
) -> impl Future<Item = (web::Data<ApiData>, String, i64), Error = ApiError> {
    let client = actix_web::client::Client::new();
    let authorisation_header = format!("Bearer {}", access_token);
    client
        .get("https://graph.microsoft.com/v1.0/me")
        .header(header::AUTHORIZATION, authorisation_header)
        .header(header::CONTENT_TYPE, ContentType::json())
        .header(header::USER_AGENT, data.user_agent())
        .send()
        .map_err(|_e| ApiError::Unwrap("failed to client.request"))
        .and_then(|response| match response.status() {
            StatusCode::OK => future::ok(response),
            _ => future::err(ApiError::Unwrap("failed to receive ok response")),
        })
        .and_then(|mut response| {
            response
                .json::<MicrosoftUser>()
                .map_err(|_e| ApiError::Unwrap("failed to parse json"))
        })
        .map(move |response| (data, response.mail, service_id))
}

fn microsoft_client(provider: Option<&ApiConfigOauth2Provider>) -> Result<BasicClient, ApiError> {
    let provider = provider.ok_or(ApiError::InvalidOauth2Provider)?;

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
