use crate::{
    grpc::{pb, util::*, Server},
    *,
};

pub async fn oauth2_url(
    server: &Server,
    request: MethodRequest<()>,
) -> MethodResult<pb::AuthOauth2UrlReply> {
    let (audit_meta, auth, _) = request.into_inner();

    let driver = server.driver();
    let args = server.options().microsoft_oauth2_args();
    blocking::<_, MethodError, _>(move || {
        let url = audit_result_err(
            driver.as_ref().as_ref(),
            audit_meta,
            AuditType::AuthMicrosoftOauth2Url,
            |driver, audit| provider_microsoft::oauth2_url(driver, audit, auth.as_ref(), &args),
        )?;
        let reply = pb::AuthOauth2UrlReply { url };
        Ok(reply)
    })
    .await
}

pub async fn oauth2_callback(
    server: &Server,
    request: MethodRequest<pb::AuthOauth2CallbackRequest>,
) -> MethodResult<pb::AuthTokenReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let client = server.client();
    let args = server.options().microsoft_oauth2_args();
    blocking::<_, MethodError, _>(move || {
        let user_token = audit_result_err(
            driver.as_ref().as_ref(),
            audit_meta,
            AuditType::AuthMicrosoftOauth2Callback,
            |driver, audit| {
                provider_microsoft::oauth2_callback(
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
    .await
}

mod provider_microsoft {
    use crate::{
        grpc::{
            methods::auth::oauth2_login, pb, util::*, ServerOptionsProvider,
            ServerProviderOauth2Args,
        },
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

    pub(crate) fn oauth2_url(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<&String>,
        args: &ServerProviderOauth2Args,
    ) -> MethodResult<String> {
        let service = key_service_authenticate(driver, audit, key_value)
            .map_err(MethodError::Unauthorised)?;

        // Microsoft Graph supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
        // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate the authorisation URL to redirect.
        let client = new_client(&service, &args.provider).map_err(MethodError::BadRequest)?;
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
            .map_err(MethodError::BadRequest)?;

        Ok(authorize_url.to_string())
    }

    pub(crate) fn oauth2_callback(
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
        let pkce_code_verifier = PkceCodeVerifier::new(csrf.value);
        let token = client
            .exchange_code(code)
            .set_pkce_verifier(pkce_code_verifier)
            .request(http_client)
            .map_err(|e| DriverError::Oauth2Request(e.into()))
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
        // TODO(refactor2): Create and keep client alive from startup.
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
