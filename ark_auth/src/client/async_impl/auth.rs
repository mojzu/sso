use crate::client::async_impl::AsyncClient;
use crate::client::Error;
use crate::server::api::{
    AuthKeyBody, AuthKeyResponse, AuthLoginBody, AuthLoginResponse, AuthOauth2UrlResponse,
    AuthPasswordMetaResponse, AuthResetPasswordBody, AuthResetPasswordConfirmBody, AuthTokenBody,
    AuthTokenPartialResponse, AuthTokenResponse,
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

        self.post("/v1/auth/provider/local/login")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| {
                res.json::<AuthLoginResponse>()
                    .map_err(|_err| Error::Unwrap)
            })
    }

    pub fn auth_local_reset_password(&self, email: &str) -> impl Future<Item = (), Error = Error> {
        let body = AuthResetPasswordBody {
            email: email.to_owned(),
            template: None,
        };

        self.post("/v1/auth/provider/local/reset/password")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
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

        self.post("/v1/auth/provider/local/reset/password/confirm")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| {
                res.json::<AuthPasswordMetaResponse>()
                    .map_err(|_err| Error::Unwrap)
            })
    }

    pub fn auth_microsoft_oauth2_request(
        &self,
    ) -> impl Future<Item = AuthOauth2UrlResponse, Error = Error> {
        self.post("/v1/auth/provider/microsoft/oauth2")
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| {
                res.json::<AuthOauth2UrlResponse>()
                    .map_err(|_err| Error::Unwrap)
            })
    }

    pub fn auth_key_verify(&self, key: &str) -> impl Future<Item = AuthKeyResponse, Error = Error> {
        let body = AuthKeyBody {
            key: key.to_owned(),
        };

        self.post("/v1/auth/key/verify")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuthKeyResponse>().map_err(|_err| Error::Unwrap))
    }

    pub fn auth_key_revoke(&self, key: &str) -> impl Future<Item = (), Error = Error> {
        let body = AuthKeyBody {
            key: key.to_owned(),
        };

        self.post("/v1/auth/key/revoke")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
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

        self.post("/v1/auth/token/verify")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| {
                res.json::<AuthTokenPartialResponse>()
                    .map_err(|_err| Error::Unwrap)
            })
    }

    pub fn auth_token_refresh(
        &self,
        token: &str,
    ) -> impl Future<Item = AuthTokenResponse, Error = Error> {
        let body = AuthTokenBody {
            token: token.to_owned(),
        };

        self.post("/v1/auth/token/refresh")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| {
                res.json::<AuthTokenResponse>()
                    .map_err(|_err| Error::Unwrap)
            })
    }

    pub fn auth_token_revoke(&self, token: &str) -> impl Future<Item = (), Error = Error> {
        let body = AuthTokenBody {
            token: token.to_owned(),
        };

        self.post("/v1/auth/token/revoke")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }
}
