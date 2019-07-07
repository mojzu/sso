use crate::client::async_impl::AsyncClient;
use crate::client::Error;
use crate::server::api::{
    route, AuthKeyBody, AuthKeyResponse, AuthLoginBody, AuthLoginResponse, AuthOauth2UrlResponse,
    AuthPasswordMetaResponse, AuthResetPasswordBody, AuthResetPasswordConfirmBody, AuthTokenBody,
    AuthTokenPartialResponse, AuthTokenResponse, AuthUpdateEmailBody, AuthUpdateEmailRevokeBody,
    AuthUpdatePasswordBody, AuthUpdatePasswordRevokeBody,
};
use futures::Future;

impl AsyncClient {
    pub fn auth_local_login(
        &self,
        email: &str,
        password: &str,
    ) -> impl Future<Item = AuthLoginResponse, Error = Error> {
        let body = AuthLoginBody {
            email: email.to_owned(),
            password: password.to_owned(),
        };

        self.post(route::AUTH_LOCAL_LOGIN)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthLoginResponse>().map_err(Into::into))
    }

    pub fn auth_local_reset_password(&self, email: &str) -> impl Future<Item = (), Error = Error> {
        let body = AuthResetPasswordBody {
            email: email.to_owned(),
            template: None,
        };

        self.post(route::AUTH_LOCAL_RESET_PASSWORD)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_local_reset_password_confirm(
        &self,
        token: &str,
        password: &str,
    ) -> impl Future<Item = AuthPasswordMetaResponse, Error = Error> {
        let body = AuthResetPasswordConfirmBody {
            token: token.to_owned(),
            password: password.to_owned(),
        };

        self.post(route::AUTH_LOCAL_RESET_PASSWORD_CONFIRM)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthPasswordMetaResponse>().map_err(Into::into))
    }

    pub fn auth_local_update_email(
        &self,
        key: Option<&str>,
        token: Option<&str>,
        password: &str,
        new_email: &str,
    ) -> impl Future<Item = (), Error = Error> {
        let body = AuthUpdateEmailBody {
            key: key.map(|x| x.to_owned()),
            token: token.map(|x| x.to_owned()),
            password: password.to_owned(),
            new_email: new_email.to_owned(),
            template: None,
        };

        self.post(route::AUTH_LOCAL_UPDATE_EMAIL)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_local_update_email_revoke(
        &self,
        token: &str,
    ) -> impl Future<Item = (), Error = Error> {
        let body = AuthUpdateEmailRevokeBody {
            token: token.to_owned(),
        };

        self.post(route::AUTH_LOCAL_UPDATE_EMAIL_REVOKE)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_local_update_password(
        &self,
        key: Option<&str>,
        token: Option<&str>,
        password: &str,
        new_password: &str,
    ) -> impl Future<Item = (), Error = Error> {
        let body = AuthUpdatePasswordBody {
            key: key.map(|x| x.to_owned()),
            token: token.map(|x| x.to_owned()),
            password: password.to_owned(),
            new_password: new_password.to_owned(),
            template: None,
        };

        self.post(route::AUTH_LOCAL_UPDATE_PASSWORD)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_local_update_password_revoke(
        &self,
        token: &str,
    ) -> impl Future<Item = (), Error = Error> {
        let body = AuthUpdatePasswordRevokeBody {
            token: token.to_owned(),
        };

        self.post(route::AUTH_LOCAL_UPDATE_PASSWORD_REVOKE)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
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

    pub fn auth_key_verify(&self, key: &str) -> impl Future<Item = AuthKeyResponse, Error = Error> {
        let body = AuthKeyBody {
            key: key.to_owned(),
        };

        self.post(route::AUTH_KEY_VERIFY)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthKeyResponse>().map_err(Into::into))
    }

    pub fn auth_key_revoke(&self, key: &str) -> impl Future<Item = (), Error = Error> {
        let body = AuthKeyBody {
            key: key.to_owned(),
        };

        self.post(route::AUTH_KEY_REVOKE)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_token_verify(
        &self,
        token: &str,
    ) -> impl Future<Item = AuthTokenPartialResponse, Error = Error> {
        let body = AuthTokenBody {
            token: token.to_owned(),
        };

        self.post(route::AUTH_TOKEN_VERIFY)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthTokenPartialResponse>().map_err(Into::into))
    }

    pub fn auth_token_refresh(
        &self,
        token: &str,
    ) -> impl Future<Item = AuthTokenResponse, Error = Error> {
        let body = AuthTokenBody {
            token: token.to_owned(),
        };

        self.post(route::AUTH_TOKEN_REFRESH)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthTokenResponse>().map_err(Into::into))
    }

    pub fn auth_token_revoke(&self, token: &str) -> impl Future<Item = (), Error = Error> {
        let body = AuthTokenBody {
            token: token.to_owned(),
        };

        self.post(route::AUTH_TOKEN_REVOKE)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }
}
