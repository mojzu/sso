use crate::{
    api::{
        AuthOauth2CallbackRequest, AuthOauth2UrlResponse, AuthProviderOauth2,
        AuthProviderOauth2Args, AuthTokenResponse, ValidateRequest,
    },
    AuditBuilder, AuditMeta, AuditType, Auth, AuthArgs, Client, CoreError, CoreOauth2Error,
    CoreResult, Csrf, Driver, Key, Service,
};
use http::header;
use oauth2::{
    basic::BasicClient, reqwest::http_client, AuthUrl, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use reqwest::Client as ReqwestClient;
use url::Url;
use uuid::Uuid;

pub fn auth_provider_github_oauth2_url(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    args: AuthProviderOauth2Args,
) -> CoreResult<AuthOauth2UrlResponse> {
    Key::authenticate_service(
        driver,
        audit_meta,
        key_value,
        AuditType::AuthGithubOauth2Url,
    )
    .and_then(|(service, mut audit)| {
        github_oauth2_url(
            driver,
            &service,
            &mut audit,
            args.provider,
            args.access_token_expires,
        )
    })
    .map(|url| AuthOauth2UrlResponse { url })
}

pub fn auth_provider_github_oauth2_callback(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    args: AuthProviderOauth2Args,
    request: AuthOauth2CallbackRequest,
) -> CoreResult<AuthTokenResponse> {
    AuthOauth2CallbackRequest::api_validate(&request)?;

    let (service, mut audit) = Key::authenticate_service(
        driver,
        audit_meta,
        key_value,
        AuditType::AuthGithubOauth2Callback,
    )?;
    let (service_id, access_token) =
        github_oauth2_callback(driver, &service, &mut audit, args.provider, request)?;
    let user_email = github_api_user_email(args.user_agent, access_token)?;
    Auth::oauth2_login(
        AuthArgs::new(driver, &service, &mut audit),
        service_id,
        user_email,
        args.access_token_expires,
        args.refresh_token_expires,
    )
    .map(|(_service, data)| AuthTokenResponse { data, audit: None })
}

fn github_oauth2_url(
    driver: &dyn Driver,
    service: &Service,
    _audit: &mut AuditBuilder,
    provider: Option<&AuthProviderOauth2>,
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

fn github_oauth2_callback(
    driver: &dyn Driver,
    service: &Service,
    _audit: &mut AuditBuilder,
    provider: Option<&AuthProviderOauth2>,
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

fn github_api_user_email(user_agent: String, access_token: String) -> CoreResult<String> {
    #[derive(Debug, Serialize, Deserialize)]
    struct GithubUser {
        email: String,
    }

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
    provider: Option<&AuthProviderOauth2>,
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
