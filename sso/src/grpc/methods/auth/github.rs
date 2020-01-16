use crate::grpc::{validate, Server};
use crate::{
    grpc::{pb, util::*},
    *,
};
use tonic::Response;
use validator::{Validate, ValidationErrors};

pub async fn oauth2_url(
    server: &Server,
    request: MethodRequest<()>,
) -> MethodResponse<pb::AuthOauth2UrlReply, MethodError> {
    let (audit_meta, auth, _) = request.into_inner();

    let driver = server.driver();
    let args = server.options().github_oauth2_args();
    let reply = blocking::<_, MethodError, _>(move || {
        let url = audit_result_err(
            driver.as_ref().as_ref(),
            audit_meta,
            AuditType::AuthGithubOauth2Url,
            |driver, audit| provider_github::oauth2_url(driver, audit, auth.as_ref(), &args),
        )?;
        let reply = pb::AuthOauth2UrlReply { url };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::AuthOauth2CallbackRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::text(e, "code", &self.code);
            validate::text(e, "state", &self.state);
        })
    }
}

pub async fn oauth2_callback(
    server: &Server,
    request: MethodRequest<pb::AuthOauth2CallbackRequest>,
) -> MethodResponse<pb::AuthTokenReply, MethodError> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let client = server.client();
    let args = server.options().github_oauth2_args();
    let reply = blocking::<_, MethodError, _>(move || {
        let user_token = audit_result_err(
            driver.as_ref().as_ref(),
            audit_meta,
            AuditType::AuthGithubOauth2Callback,
            |driver, audit| {
                provider_github::oauth2_callback(
                    driver,
                    audit,
                    auth.as_ref(),
                    &args,
                    &req,
                    client.as_ref(),
                )
            },
        )?;
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

mod provider_github {
    use crate::{
        grpc::{
            methods::auth::oauth2_login, pb, util::*, ServerOptionsProvider,
            ServerProviderOauth2Args,
        },
        pattern::*,
        AuditBuilder, CsrfCreate, Driver, DriverError, DriverResult, Service, UserToken,
    };
    use oauth2::{
        basic::BasicClient, reqwest::http_client, AuthUrl, AuthorizationCode, ClientId,
        ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
    };
    use reqwest::Client as SyncClient;
    use url::Url;

    pub fn oauth2_url(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<&String>,
        args: &ServerProviderOauth2Args,
    ) -> MethodResult<String> {
        let service = key_service_authenticate(driver, audit, key_value)
            .map_err(MethodError::Unauthorised)?;

        // Generate the authorisation URL to which we'll redirect the user.
        let client = new_client(&service, &args.provider).map_err(MethodError::BadRequest)?;
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
            .map_err(MethodError::BadRequest)?;

        Ok(authorise_url.to_string())
    }

    pub fn oauth2_callback(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<&String>,
        args: &ServerProviderOauth2Args,
        request: &pb::AuthOauth2CallbackRequest,
        client_sync: &SyncClient,
    ) -> MethodResult<UserToken> {
        let service = key_service_authenticate(driver, audit, key_value)
            .map_err(MethodError::Unauthorised)?;

        // Read the CSRF key using state value, rebuild code verifier from value.
        let csrf = driver
            .csrf_read(&request.state)
            .map_err(MethodError::BadRequest)?
            .ok_or_else(|| DriverError::CsrfNotFoundOrUsed)
            .map_err(MethodError::BadRequest)?;

        // Exchange the code with a token.
        let client = new_client(&service, &args.provider).map_err(MethodError::BadRequest)?;
        let code = AuthorizationCode::new(request.code.clone());
        let token = client
            .exchange_code(code)
            .request(http_client)
            .map_err(|err| DriverError::Oauth2Request(err.into()))
            .map_err(MethodError::BadRequest)?;

        // Return access token value.
        let (service_id, access_token) =
            (csrf.service_id, token.access_token().secret().to_owned());

        let user_email =
            api_user_email(client_sync, access_token).map_err(MethodError::BadRequest)?;
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
            .header("authorization", authorisation)
            .send()
            .and_then(|res| res.error_for_status())
            .and_then(|mut res| res.json::<GithubUser>())
            .map_err(DriverError::Reqwest)
            .map(|res| res.email)
    }

    fn new_client(
        service: &Service,
        provider: &Option<ServerOptionsProvider>,
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
