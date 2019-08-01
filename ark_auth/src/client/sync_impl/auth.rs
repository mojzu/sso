use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::api::{
    route, AuditCustom, AuthKeyBody, AuthKeyResponse, AuthLoginBody, AuthLoginResponse,
    AuthOauth2UrlResponse, AuthPasswordMetaResponse, AuthResetPasswordBody,
    AuthResetPasswordConfirmBody, AuthTokenBody, AuthTokenPartialResponse, AuthTokenResponse,
    AuthUpdateEmailBody, AuthUpdatePasswordBody,
};

impl SyncClient {
    pub fn auth_local_login<T1, T2>(
        &self,
        email: T1,
        password: T2,
    ) -> Result<AuthLoginResponse, Error>
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        let body = AuthLoginBody {
            email: email.into(),
            password: password.into(),
        };
        self.post_json(route::AUTH_LOCAL_LOGIN, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthLoginResponse>().map_err(Into::into))
    }

    pub fn auth_local_reset_password<T1>(&self, email: T1) -> Result<(), Error>
    where
        T1: Into<String>,
    {
        let body = AuthResetPasswordBody {
            email: email.into(),
        };
        self.post_json(route::AUTH_LOCAL_RESET_PASSWORD, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_local_reset_password_confirm<T1, T2>(
        &self,
        token: T1,
        password: T2,
    ) -> Result<AuthPasswordMetaResponse, Error>
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        let body = AuthResetPasswordConfirmBody {
            token: token.into(),
            password: password.into(),
        };
        self.post_json(route::AUTH_LOCAL_RESET_PASSWORD_CONFIRM, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthPasswordMetaResponse>().map_err(Into::into))
    }

    pub fn auth_local_update_email(&self, body: AuthUpdateEmailBody) -> Result<(), Error> {
        self.post_json(route::AUTH_LOCAL_UPDATE_EMAIL, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_local_update_email_revoke(&self, body: AuthTokenBody) -> Result<(), Error> {
        self.post_json(route::AUTH_LOCAL_UPDATE_EMAIL_REVOKE, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_local_update_password(
        &self,
        body: AuthUpdatePasswordBody,
    ) -> Result<AuthPasswordMetaResponse, Error> {
        self.post_json(route::AUTH_LOCAL_UPDATE_PASSWORD, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthPasswordMetaResponse>().map_err(Into::into))
    }

    pub fn auth_local_update_password_revoke(&self, body: AuthTokenBody) -> Result<(), Error> {
        self.post_json(route::AUTH_LOCAL_UPDATE_PASSWORD_REVOKE, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_github_oauth2_request(&self) -> Result<AuthOauth2UrlResponse, Error> {
        self.post(route::AUTH_GITHUB_OAUTH2)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthOauth2UrlResponse>().map_err(Into::into))
    }

    pub fn auth_microsoft_oauth2_request(&self) -> Result<AuthOauth2UrlResponse, Error> {
        self.post(route::AUTH_MICROSOFT_OAUTH2)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthOauth2UrlResponse>().map_err(Into::into))
    }

    pub fn auth_key_verify<T1>(
        &self,
        key: T1,
        audit: Option<AuditCustom>,
    ) -> Result<AuthKeyResponse, Error>
    where
        T1: Into<String>,
    {
        let body = AuthKeyBody {
            key: key.into(),
            audit,
        };
        self.post_json(route::AUTH_KEY_VERIFY, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthKeyResponse>().map_err(Into::into))
    }

    pub fn auth_key_revoke<T1>(&self, key: T1, audit: Option<AuditCustom>) -> Result<(), Error>
    where
        T1: Into<String>,
    {
        let body = AuthKeyBody {
            key: key.into(),
            audit,
        };
        self.post_json(route::AUTH_KEY_REVOKE, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_token_verify<T1>(
        &self,
        token: T1,
        audit: Option<AuditCustom>,
    ) -> Result<AuthTokenPartialResponse, Error>
    where
        T1: Into<String>,
    {
        let body = AuthTokenBody {
            token: token.into(),
            audit,
        };
        self.post_json(route::AUTH_TOKEN_VERIFY, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthTokenPartialResponse>().map_err(Into::into))
    }

    pub fn auth_token_refresh<T1>(
        &self,
        token: T1,
        audit: Option<AuditCustom>,
    ) -> Result<AuthTokenResponse, Error>
    where
        T1: Into<String>,
    {
        let body = AuthTokenBody {
            token: token.into(),
            audit,
        };
        self.post_json(route::AUTH_TOKEN_REFRESH, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthTokenResponse>().map_err(Into::into))
    }

    pub fn auth_token_revoke<T1>(&self, token: T1, audit: Option<AuditCustom>) -> Result<(), Error>
    where
        T1: Into<String>,
    {
        let body = AuthTokenBody {
            token: token.into(),
            audit,
        };
        self.post_json(route::AUTH_TOKEN_REVOKE, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .map(|_res| ())
    }
}
