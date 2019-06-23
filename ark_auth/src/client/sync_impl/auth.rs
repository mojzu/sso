use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::route::auth::provider::local::{LoginBody, LoginResponse, ResetPasswordBody};
use crate::server::route::auth::provider::Oauth2UrlResponse;
use crate::server::route::auth::{
    KeyBody, KeyResponse, TokenBody, TokenPartialResponse, TokenResponse,
};
use actix_web::http::StatusCode;

impl SyncClient {
    pub fn auth_local_login(&self, email: &str, password: &str) -> Result<LoginResponse, Error> {
        let body = LoginBody {
            email: email.to_owned(),
            password: password.to_owned(),
        };

        self.post_json("/v1/auth/provider/local/login", &body)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<LoginResponse>().map_err(|_err| Error::Unwrap))
    }

    pub fn auth_local_reset_password(&self, email: &str) -> Result<(), Error> {
        let body = ResetPasswordBody {
            email: email.to_owned(),
            template: None,
        };

        self.post_json("/v1/auth/provider/local/reset/password", &body)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => Ok(()),
                _ => Err(Error::Unwrap),
            })
    }

    pub fn auth_microsoft_oauth2_request(&self) -> Result<Oauth2UrlResponse, Error> {
        self.post("/v1/auth/provider/microsoft/oauth2")
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| {
                res.json::<Oauth2UrlResponse>()
                    .map_err(|_err| Error::Unwrap)
            })
    }

    pub fn auth_key_verify(&self, key: &str) -> Result<KeyResponse, Error> {
        let body = KeyBody {
            key: key.to_owned(),
        };

        self.post_json("/v1/auth/key/verify", &body)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<KeyResponse>().map_err(|_err| Error::Unwrap))
    }

    pub fn auth_key_revoke(&self, key: &str) -> Result<(), Error> {
        let body = KeyBody {
            key: key.to_owned(),
        };

        self.post_json("/v1/auth/key/revoke", &body)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .map(|_res| ())
    }

    pub fn auth_token_verify(&self, token: &str) -> Result<TokenPartialResponse, Error> {
        let body = TokenBody {
            token: token.to_owned(),
        };

        self.post_json("/v1/auth/token/verify", &body)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| {
                res.json::<TokenPartialResponse>()
                    .map_err(|_err| Error::Unwrap)
            })
    }

    pub fn auth_token_refresh(&self, token: &str) -> Result<TokenResponse, Error> {
        let body = TokenBody {
            token: token.to_owned(),
        };

        self.post_json("/v1/auth/token/refresh", &body)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<TokenResponse>().map_err(|_err| Error::Unwrap))
    }

    pub fn auth_token_revoke(&self, token: &str) -> Result<(), Error> {
        let body = TokenBody {
            token: token.to_owned(),
        };

        self.post_json("/v1/auth/token/revoke", &body)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .map(|_res| ())
    }
}
