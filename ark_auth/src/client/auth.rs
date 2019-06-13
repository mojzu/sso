use crate::client::{Client, ClientError};
use crate::server::route::auth::reset::PasswordBody;
use crate::server::route::auth::{KeyBody, KeyResponse};
use crate::server::route::auth::{LoginBody, LoginResponse};
use actix_web::http::StatusCode;
use futures::{future, Future};

impl Client {
    pub fn auth_login(
        &self,
        email: &str,
        password: &str,
    ) -> impl Future<Item = LoginResponse, Error = ClientError> {
        let body = LoginBody {
            email: email.to_owned(),
            password: password.to_owned(),
        };

        self.post("/v1/auth/login")
            .send_json(&body)
            .map_err(|_err| ClientError::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => future::ok(res),
                _ => future::err(ClientError::Unwrap),
            })
            .and_then(|mut res| {
                res.json::<LoginResponse>()
                    .map_err(|_err| ClientError::Unwrap)
            })
    }

    pub fn auth_reset_password(&self, email: &str) -> impl Future<Item = (), Error = ClientError> {
        let body = PasswordBody {
            email: email.to_owned(),
            template: None,
        };

        self.post("/v1/auth/reset/password")
            .send_json(&body)
            .map_err(|_err| ClientError::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => future::ok(()),
                _ => future::err(ClientError::Unwrap),
            })
    }

    pub fn auth_key_verify(
        &self,
        key: &str,
    ) -> impl Future<Item = KeyResponse, Error = ClientError> {
        let body = KeyBody {
            key: key.to_owned(),
        };

        self.post("/v1/auth/key/verify")
            .send_json(&body)
            .map_err(|_err| ClientError::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => future::ok(res),
                _ => future::err(ClientError::Unwrap),
            })
            .and_then(|mut res| {
                res.json::<KeyResponse>()
                    .map_err(|_err| ClientError::Unwrap)
            })
    }
}
