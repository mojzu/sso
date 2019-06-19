use crate::client::async_impl::AsyncClient;
use crate::client::Error;
use crate::server::route::auth::provider::local::{LoginBody, LoginResponse, ResetPasswordBody};
use crate::server::route::auth::{KeyBody, KeyResponse, TokenBody, TokenResponse};
use futures::Future;

impl AsyncClient {
    pub fn auth_local_login(
        &self,
        email: &str,
        password: &str,
    ) -> impl Future<Item = LoginResponse, Error = Error> {
        let body = LoginBody {
            email: email.to_owned(),
            password: password.to_owned(),
        };

        self.post("/v1/auth/provider/local/login")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<LoginResponse>().map_err(|_err| Error::Unwrap))
    }

    pub fn auth_local_reset_password(&self, email: &str) -> impl Future<Item = (), Error = Error> {
        let body = ResetPasswordBody {
            email: email.to_owned(),
            template: None,
        };

        self.post("/v1/auth/provider/local/reset/password")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_key_verify(&self, key: &str) -> impl Future<Item = KeyResponse, Error = Error> {
        let body = KeyBody {
            key: key.to_owned(),
        };

        self.post("/v1/auth/key/verify")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<KeyResponse>().map_err(|_err| Error::Unwrap))
    }

    pub fn auth_key_revoke(&self, key: &str) -> impl Future<Item = (), Error = Error> {
        let body = KeyBody {
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
    ) -> impl Future<Item = TokenResponse, Error = Error> {
        let body = TokenBody {
            token: token.to_owned(),
        };

        self.post("/v1/auth/token/verify")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<TokenResponse>().map_err(|_err| Error::Unwrap))
    }

    pub fn auth_token_refresh(
        &self,
        token: &str,
    ) -> impl Future<Item = TokenResponse, Error = Error> {
        let body = TokenBody {
            token: token.to_owned(),
        };

        self.post("/v1/auth/token/refresh")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<TokenResponse>().map_err(|_err| Error::Unwrap))
    }

    pub fn auth_token_revoke(&self, token: &str) -> impl Future<Item = (), Error = Error> {
        let body = TokenBody {
            token: token.to_owned(),
        };

        self.post("/v1/auth/token/revoke")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }
}
