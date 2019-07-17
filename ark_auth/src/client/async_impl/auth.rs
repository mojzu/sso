use crate::client::async_impl::AsyncClient;
use crate::client::Error;
use crate::server::api::{
    route, AuthKeyBody, AuthKeyResponse, AuthLoginBody, AuthLoginResponse, AuthOauth2UrlResponse,
    AuthPasswordMetaResponse, AuthResetPasswordBody, AuthResetPasswordConfirmBody, AuthTokenBody,
    AuthTokenPartialResponse, AuthTokenResponse, AuthUpdateEmailBody, AuthUpdatePasswordBody,
};
use futures::Future;

impl AsyncClient {
    pub fn auth_local_login<T1, T2>(
        &self,
        email: T1,
        password: T2,
    ) -> impl Future<Item = AuthLoginResponse, Error = Error>
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        let body = AuthLoginBody {
            email: email.into(),
            password: password.into(),
        };
        self.post(route::AUTH_LOCAL_LOGIN)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthLoginResponse>().map_err(Into::into))
    }

    pub fn auth_local_reset_password<T1>(&self, email: T1) -> impl Future<Item = (), Error = Error>
    where
        T1: Into<String>,
    {
        let body = AuthResetPasswordBody {
            email: email.into(),
        };
        self.post(route::AUTH_LOCAL_RESET_PASSWORD)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_local_reset_password_confirm<T1, T2>(
        &self,
        token: T1,
        password: T2,
    ) -> impl Future<Item = AuthPasswordMetaResponse, Error = Error>
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        let body = AuthResetPasswordConfirmBody {
            token: token.into(),
            password: password.into(),
        };
        self.post(route::AUTH_LOCAL_RESET_PASSWORD_CONFIRM)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthPasswordMetaResponse>().map_err(Into::into))
    }

    pub fn auth_local_update_email(
        &self,
        key: Option<String>,
        token: Option<String>,
        password: String,
        new_email: String,
    ) -> impl Future<Item = (), Error = Error> {
        let body = AuthUpdateEmailBody {
            key,
            token,
            password,
            new_email,
        };
        self.post(route::AUTH_LOCAL_UPDATE_EMAIL)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_local_update_email_revoke<T1>(
        &self,
        token: T1,
    ) -> impl Future<Item = (), Error = Error>
    where
        T1: Into<String>,
    {
        let body = AuthTokenBody {
            token: token.into(),
        };
        self.post(route::AUTH_LOCAL_UPDATE_EMAIL_REVOKE)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_local_update_password(
        &self,
        key: Option<String>,
        token: Option<String>,
        password: String,
        new_password: String,
    ) -> impl Future<Item = (), Error = Error> {
        let body = AuthUpdatePasswordBody {
            key,
            token,
            password,
            new_password,
        };
        self.post(route::AUTH_LOCAL_UPDATE_PASSWORD)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_local_update_password_revoke<T1>(
        &self,
        token: T1,
    ) -> impl Future<Item = (), Error = Error>
    where
        T1: Into<String>,
    {
        let body = AuthTokenBody {
            token: token.into(),
        };
        self.post(route::AUTH_LOCAL_UPDATE_PASSWORD_REVOKE)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_github_oauth2_request(
        &self,
    ) -> impl Future<Item = AuthOauth2UrlResponse, Error = Error> {
        self.post(route::AUTH_GITHUB_OAUTH2)
            .send()
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthOauth2UrlResponse>().map_err(Into::into))
    }

    pub fn auth_microsoft_oauth2_request(
        &self,
    ) -> impl Future<Item = AuthOauth2UrlResponse, Error = Error> {
        self.post(route::AUTH_MICROSOFT_OAUTH2)
            .send()
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthOauth2UrlResponse>().map_err(Into::into))
    }

    pub fn auth_key_verify<T1>(&self, key: T1) -> impl Future<Item = AuthKeyResponse, Error = Error>
    where
        T1: Into<String>,
    {
        let body = AuthKeyBody { key: key.into() };
        self.post(route::AUTH_KEY_VERIFY)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthKeyResponse>().map_err(Into::into))
    }

    pub fn auth_key_revoke<T1>(&self, key: T1) -> impl Future<Item = (), Error = Error>
    where
        T1: Into<String>,
    {
        let body = AuthKeyBody { key: key.into() };
        self.post(route::AUTH_KEY_REVOKE)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_token_verify<T1>(
        &self,
        token: T1,
    ) -> impl Future<Item = AuthTokenPartialResponse, Error = Error>
    where
        T1: Into<String>,
    {
        let body = AuthTokenBody {
            token: token.into(),
        };
        self.post(route::AUTH_TOKEN_VERIFY)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthTokenPartialResponse>().map_err(Into::into))
    }

    pub fn auth_token_refresh<T1>(
        &self,
        token: T1,
    ) -> impl Future<Item = AuthTokenResponse, Error = Error>
    where
        T1: Into<String>,
    {
        let body = AuthTokenBody {
            token: token.into(),
        };
        self.post(route::AUTH_TOKEN_REFRESH)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthTokenResponse>().map_err(Into::into))
    }

    pub fn auth_token_revoke<T1>(&self, token: T1) -> impl Future<Item = (), Error = Error>
    where
        T1: Into<String>,
    {
        let body = AuthTokenBody {
            token: token.into(),
        };
        self.post(route::AUTH_TOKEN_REVOKE)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }
}
