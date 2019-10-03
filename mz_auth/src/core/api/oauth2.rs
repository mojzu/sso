use crate::{
    api_types::AuthOauth2CallbackRequest, ApiProviderOauth2, AuditBuilder, Client, CoreError,
    CoreOauth2Error, CoreResult, Csrf, Driver, Service,
};
use http::header;
use oauth2::{
    basic::BasicClient, reqwest::http_client, AuthType, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::Client as ReqwestClient;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct GithubUser {
    email: String,
}

pub fn github_oauth2_url(
    driver: &dyn Driver,
    service: &Service,
    _audit: &mut AuditBuilder,
    provider: Option<&ApiProviderOauth2>,
    access_token_expires: i64,
) -> CoreResult<String> {
    // Generate the authorisation URL to which we'll redirect the user.
    let client = github_client(service, provider)?;
    let (authorise_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    // Save the state and code verifier secrets as a CSRF key, value.
    let csrf_key = csrf_state.secret();
    Csrf::create(
        driver,
        service,
        String::from(csrf_key),
        String::from(csrf_key),
        access_token_expires,
    )?;

    Ok(authorise_url.to_string())
}

pub fn github_oauth2_callback(
    driver: &dyn Driver,
    service: &Service,
    _audit: &mut AuditBuilder,
    provider: Option<&ApiProviderOauth2>,
    request: AuthOauth2CallbackRequest,
) -> CoreResult<(Uuid, String)> {
    // Read the CSRF key using state value, rebuild code verifier from value.
    let csrf = Csrf::read_opt(driver, request.state)?;
    let csrf = csrf.ok_or_else(|| CoreError::Oauth2(CoreOauth2Error::Csrf))?;

    // Exchange the code with a token.
    let client = github_client(service, provider)?;
    let code = AuthorizationCode::new(request.code);
    let token = client
        .exchange_code(code)
        .request(http_client)
        .map_err(|err| CoreError::Oauth2(CoreOauth2Error::Oauth2Request(err.into())))?;

    // Return access token value.
    Ok((csrf.service_id, token.access_token().secret().to_owned()))
}

pub fn github_api_user_email(user_agent: String, access_token: String) -> CoreResult<String> {
    let authorisation = format!("token {}", access_token);
    let client = ReqwestClient::builder().use_rustls_tls().build().unwrap();

    client
        .get("https://api.github.com/user")
        .header(header::USER_AGENT, user_agent)
        .header(header::AUTHORIZATION, authorisation)
        .send()
        .map_err(Into::into)
        .and_then(Client::response_json::<GithubUser>)
        .map_err(Into::into)
        .map(|res| res.email)
}

fn github_client(
    service: &Service,
    provider: Option<&ApiProviderOauth2>,
) -> CoreResult<BasicClient> {
    let (provider_github_oauth2_url, provider) =
        match (&service.provider_github_oauth2_url, provider) {
            (Some(provider_github_oauth2_url), Some(provider)) => {
                Ok((provider_github_oauth2_url, provider))
            }
            _ => {
                // Warn OAuth2 is disabled, return bad request error so internal server error
                // is not returned to the client.
                let err = CoreError::Oauth2(CoreOauth2Error::Disabled);
                warn!("{}", err);
                Err(CoreError::BadRequest)
            }
        }?;

    let github_client_id = ClientId::new(provider.client_id.to_owned());
    let github_client_secret = ClientSecret::new(provider.client_secret.to_owned());

    let auth_url =
        Url::parse("https://github.com/login/oauth/authorize").map_err(CoreError::UrlParse)?;
    let auth_url = AuthUrl::new(auth_url);
    let token_url =
        Url::parse("https://github.com/login/oauth/access_token").map_err(CoreError::UrlParse)?;
    let token_url = TokenUrl::new(token_url);

    let redirect_url = Url::parse(&provider_github_oauth2_url).map_err(CoreError::UrlParse)?;
    Ok(BasicClient::new(
        github_client_id,
        Some(github_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_url(RedirectUrl::new(redirect_url)))
}

#[derive(Debug, Serialize, Deserialize)]
struct MicrosoftUser {
    mail: String,
}

pub fn microsoft_oauth2_url(
    driver: &dyn Driver,
    service: &Service,
    _audit: &mut AuditBuilder,
    provider: Option<&ApiProviderOauth2>,
    access_token_expires: i64,
) -> CoreResult<String> {
    // Microsoft Graph supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorisation URL to redirect.
    let client = microsoft_client(service, provider)?;
    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://graph.microsoft.com/User.Read".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    // Save the state and code verifier secrets as a CSRF key, value.
    let csrf_key = csrf_state.secret();
    let csrf_value = pkce_code_verifier.secret();
    Csrf::create(
        driver,
        service,
        String::from(csrf_key),
        String::from(csrf_value),
        access_token_expires,
    )?;

    Ok(authorize_url.to_string())
}

pub fn microsoft_oauth2_callback(
    driver: &dyn Driver,
    service: &Service,
    _audit: &mut AuditBuilder,
    provider: Option<&ApiProviderOauth2>,
    request: AuthOauth2CallbackRequest,
) -> CoreResult<(Uuid, String)> {
    // Read the CSRF key using state value, rebuild code verifier from value.
    let csrf = Csrf::read_opt(driver, request.state)?;
    let csrf = csrf.ok_or_else(|| CoreError::Oauth2(CoreOauth2Error::Csrf))?;

    // Exchange the code with a token.
    let client = microsoft_client(service, provider)?;
    let code = AuthorizationCode::new(request.code);
    let pkce_code_verifier = PkceCodeVerifier::new(csrf.value);
    let token = client
        .exchange_code(code)
        .set_pkce_verifier(pkce_code_verifier)
        .request(http_client)
        .map_err(|err| CoreError::Oauth2(CoreOauth2Error::Oauth2Request(err.into())))?;

    // Return access token value.
    Ok((csrf.service_id, token.access_token().secret().to_owned()))
}

pub fn microsoft_api_user_email(user_agent: String, access_token: String) -> CoreResult<String> {
    let authorisation = format!("Bearer {}", access_token);
    let client = ReqwestClient::builder().use_rustls_tls().build().unwrap();

    client
        .get("https://graph.microsoft.com/v1.0/me")
        .header(header::USER_AGENT, user_agent)
        .header(header::AUTHORIZATION, authorisation)
        .send()
        .map_err(Into::into)
        .and_then(Client::response_json::<MicrosoftUser>)
        .map_err(Into::into)
        .map(|res| res.mail)
}

fn microsoft_client(
    service: &Service,
    provider: Option<&ApiProviderOauth2>,
) -> CoreResult<BasicClient> {
    let (provider_microsoft_oauth2_url, provider) =
        match (&service.provider_microsoft_oauth2_url, provider) {
            (Some(provider_microsoft_oauth2_url), Some(provider)) => {
                Ok((provider_microsoft_oauth2_url, provider))
            }
            _ => {
                // Warn OAuth2 is disabled, return bad request error so internal server error
                // is not returned to the client.
                let err = CoreError::Oauth2(CoreOauth2Error::Disabled);
                warn!("{}", err);
                Err(CoreError::BadRequest)
            }
        }?;

    let graph_client_id = ClientId::new(provider.client_id.to_owned());
    let graph_client_secret = ClientSecret::new(provider.client_secret.to_owned());

    let auth_url = Url::parse("https://login.microsoftonline.com/common/oauth2/v2.0/authorize")
        .map_err(CoreError::UrlParse)?;
    let auth_url = AuthUrl::new(auth_url);
    let token_url = Url::parse("https://login.microsoftonline.com/common/oauth2/v2.0/token")
        .map_err(CoreError::UrlParse)?;
    let token_url = TokenUrl::new(token_url);

    let redirect_url = Url::parse(&provider_microsoft_oauth2_url).map_err(CoreError::UrlParse)?;
    Ok(BasicClient::new(
        graph_client_id,
        Some(graph_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_auth_type(AuthType::RequestBody)
    .set_redirect_url(RedirectUrl::new(redirect_url)))
}
