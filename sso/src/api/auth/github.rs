use crate::{
    api::{
        result_audit, result_audit_err, ApiResult, AuthOauth2CallbackRequest,
        AuthOauth2UrlResponse, AuthProviderOauth2Args, AuthTokenResponse, ValidateRequest,
    },
    AuditBuilder, AuditMeta, AuditType, Driver,
};
use reqwest::Client as SyncClient;

pub fn auth_provider_github_oauth2_url(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    args: AuthProviderOauth2Args,
) -> ApiResult<AuthOauth2UrlResponse> {
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthGithubOauth2Url);

    let res = provider_github::oauth2_url(driver, &mut audit, key_value, &args);
    result_audit_err(driver, &audit, res).map(|url| AuthOauth2UrlResponse { url })
}

pub fn auth_provider_github_oauth2_callback(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuthOauth2CallbackRequest,
    args: AuthProviderOauth2Args,
    client_sync: &SyncClient,
) -> ApiResult<AuthTokenResponse> {
    AuthOauth2CallbackRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthGithubOauth2Callback);

    let res = provider_github::oauth2_callback(
        driver,
        &mut audit,
        key_value,
        &args,
        request,
        client_sync,
    );
    result_audit(driver, &audit, res).map(|data| AuthTokenResponse { data, audit: None })
}

mod provider_github {
    use crate::{
        api::{
            auth::server_auth::oauth2_login, ApiError, ApiResult, AuthOauth2CallbackRequest,
            AuthProviderOauth2, AuthProviderOauth2Args,
        },
        pattern::*,
        AuditBuilder, Client, CsrfCreate, Driver, DriverError, DriverResult, Service, UserToken,
    };
    use http::header;
    use oauth2::{
        basic::BasicClient, reqwest::http_client, AuthUrl, AuthorizationCode, ClientId,
        ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
    };
    use reqwest::Client as SyncClient;
    use url::Url;

    pub fn oauth2_url(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        args: &AuthProviderOauth2Args,
    ) -> ApiResult<String> {
        let service =
            key_service_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Generate the authorisation URL to which we'll redirect the user.
        let client = new_client(&service, args.provider).map_err(ApiError::BadRequest)?;
        let (authorise_url, csrf_state) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("user:email".to_string()))
            .url();

        // Save the state and code verifier secrets as a CSRF key, value.
        let csrf_key = csrf_state.secret();
        let csrf_create =
            CsrfCreate::new(csrf_key, csrf_key, args.access_token_expires, service.id);
        driver
            .csrf_create(&csrf_create)
            .map_err(ApiError::BadRequest)?;

        Ok(authorise_url.to_string())
    }

    pub fn oauth2_callback(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        args: &AuthProviderOauth2Args,
        request: AuthOauth2CallbackRequest,
        client_sync: &SyncClient,
    ) -> ApiResult<UserToken> {
        let service =
            key_service_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Read the CSRF key using state value, rebuild code verifier from value.
        let csrf = driver
            .csrf_read(&request.state)
            .map_err(ApiError::BadRequest)?
            .ok_or_else(|| DriverError::CsrfNotFoundOrUsed)
            .map_err(ApiError::BadRequest)?;

        // Exchange the code with a token.
        let client = new_client(&service, args.provider).map_err(ApiError::BadRequest)?;
        let code = AuthorizationCode::new(request.code);
        let token = client
            .exchange_code(code)
            .request(http_client)
            .map_err(|err| DriverError::Oauth2Request(err.into()))
            .map_err(ApiError::BadRequest)?;

        // Return access token value.
        let (service_id, access_token) =
            (csrf.service_id, token.access_token().secret().to_owned());

        let user_email = api_user_email(client_sync, access_token).map_err(ApiError::BadRequest)?;
        oauth2_login(
            driver,
            audit,
            &service,
            service_id,
            user_email,
            args.access_token_expires,
            args.refresh_token_expires,
        )
    }

    fn api_user_email(client: &SyncClient, access_token: String) -> DriverResult<String> {
        #[derive(Debug, Serialize, Deserialize)]
        struct GithubUser {
            email: String,
        }

        let authorisation = format!("token {}", access_token);
        client
            .get("https://api.github.com/user")
            .header(header::AUTHORIZATION, authorisation)
            .send()
            .map_err(Into::into)
            .and_then(Client::response_json::<GithubUser>)
            .map_err(Into::into)
            .map(|res| res.email)
    }

    fn new_client(
        service: &Service,
        provider: Option<&AuthProviderOauth2>,
    ) -> DriverResult<BasicClient> {
        let (provider_github_oauth2_url, provider) =
            match (&service.provider_github_oauth2_url, provider) {
                (Some(provider_github_oauth2_url), Some(provider)) => {
                    Ok((provider_github_oauth2_url, provider))
                }
                _ => Err(DriverError::ServiceProviderGithubOauth2Disabled),
            }?;

        let new_client_id = ClientId::new(provider.client_id.to_owned());
        let new_client_secret = ClientSecret::new(provider.client_secret.to_owned());

        let auth_url = Url::parse("https://github.com/login/oauth/authorize")
            .map_err(DriverError::UrlParse)?;
        let auth_url = AuthUrl::new(auth_url);
        let token_url = Url::parse("https://github.com/login/oauth/access_token")
            .map_err(DriverError::UrlParse)?;
        let token_url = TokenUrl::new(token_url);

        let redirect_url =
            Url::parse(&provider_github_oauth2_url).map_err(DriverError::UrlParse)?;
        Ok(BasicClient::new(
            new_client_id,
            Some(new_client_secret),
            auth_url,
            Some(token_url),
        )
        .set_redirect_url(RedirectUrl::new(redirect_url)))
    }
}
