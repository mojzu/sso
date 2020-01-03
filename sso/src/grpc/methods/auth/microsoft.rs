use crate::{
    api::{self},
    grpc::{pb, util::*, ServerProviderOauth2Args},
    *,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub async fn oauth2_url(
    driver: Arc<Box<dyn Driver>>,
    request: Request<()>,
    args: ServerProviderOauth2Args,
) -> Result<Response<pb::AuthOauth2UrlReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthMicrosoftOauth2Url);
        let res: Result<String, Status> =
            { provider_microsoft::oauth2_url(driver.as_ref().as_ref(), &mut audit, auth, &args) };
        let url = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuthOauth2UrlReply { url };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn oauth2_callback(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::AuthOauth2CallbackRequest>,
    args: ServerProviderOauth2Args,
    client: Arc<reqwest::Client>,
) -> Result<Response<pb::AuthTokenReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req = request.into_inner();
    // TODO(refactor): Validate input.
    // AuditList::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthMicrosoftOauth2Callback);
        let res: Result<UserToken, Status> = {
            provider_microsoft::oauth2_callback(
                driver.as_ref().as_ref(),
                &mut audit,
                auth,
                &args,
                req,
                client.as_ref(),
            )
        };
        let user_token = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuthTokenReply {
            user: Some(user_token.user.clone().into()),
            access: Some(user_token.access_token()),
            refresh: Some(user_token.refresh_token()),
            audit: None,
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

mod provider_microsoft {
    use crate::{
        api::{ApiError, ApiResult},
        grpc::{methods::auth::oauth2_login, pb, ServerOptionsProvider, ServerProviderOauth2Args},
        pattern::*,
        AuditBuilder, CsrfCreate, Driver, DriverError, DriverResult, Service, UserToken,
    };
    use oauth2::{
        basic::BasicClient, reqwest::http_client, AuthType, AuthUrl, AuthorizationCode, ClientId,
        ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
        TokenResponse, TokenUrl,
    };
    use reqwest::Client as SyncClient;
    use url::Url;

    pub fn oauth2_url(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        args: &ServerProviderOauth2Args,
    ) -> ApiResult<String> {
        let service =
            key_service_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Microsoft Graph supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
        // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate the authorisation URL to redirect.
        let client = new_client(&service, &args.provider).map_err(ApiError::BadRequest)?;
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
        let csrf_create =
            CsrfCreate::new(csrf_key, csrf_value, args.access_token_expires, service.id);
        driver
            .csrf_create(&csrf_create)
            .map_err(ApiError::BadRequest)?;

        Ok(authorize_url.to_string())
    }

    pub fn oauth2_callback(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        args: &ServerProviderOauth2Args,
        request: pb::AuthOauth2CallbackRequest,
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
        let client = new_client(&service, &args.provider).map_err(ApiError::BadRequest)?;
        let code = AuthorizationCode::new(request.code);
        let pkce_code_verifier = PkceCodeVerifier::new(csrf.value);
        let token = client
            .exchange_code(code)
            .set_pkce_verifier(pkce_code_verifier)
            .request(http_client)
            .map_err(|e| DriverError::Oauth2Request(e.into()))
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
        struct MicrosoftUser {
            mail: String,
        }

        let authorisation = format!("Bearer {}", access_token);
        client
            .get("https://graph.microsoft.com/v1.0/me")
            .header("authorization", authorisation)
            .send()
            .and_then(|res| res.error_for_status())
            .and_then(|mut res| res.json::<MicrosoftUser>())
            .map_err(DriverError::Reqwest)
            .map(|res| res.mail)
    }

    fn new_client(
        service: &Service,
        provider: &Option<ServerOptionsProvider>,
    ) -> DriverResult<BasicClient> {
        let (provider_microsoft_oauth2_url, provider) =
            match (&service.provider_microsoft_oauth2_url, provider) {
                (Some(provider_microsoft_oauth2_url), Some(provider)) => {
                    Ok((provider_microsoft_oauth2_url, provider))
                }
                _ => Err(DriverError::ServiceProviderMicrosoftOauth2Disabled),
            }?;

        let graph_client_id = ClientId::new(provider.client_id.to_owned());
        let graph_client_secret = ClientSecret::new(provider.client_secret.to_owned());

        let auth_url = Url::parse("https://login.microsoftonline.com/common/oauth2/v2.0/authorize")
            .map_err(DriverError::UrlParse)?;
        let auth_url = AuthUrl::new(auth_url);
        let token_url = Url::parse("https://login.microsoftonline.com/common/oauth2/v2.0/token")
            .map_err(DriverError::UrlParse)?;
        let token_url = TokenUrl::new(token_url);

        let redirect_url =
            Url::parse(&provider_microsoft_oauth2_url).map_err(DriverError::UrlParse)?;
        Ok(BasicClient::new(
            graph_client_id,
            Some(graph_client_secret),
            auth_url,
            Some(token_url),
        )
        .set_auth_type(AuthType::RequestBody)
        .set_redirect_url(RedirectUrl::new(redirect_url)))
    }
}