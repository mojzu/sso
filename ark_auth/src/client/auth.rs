use crate::client::{Client, ClientError};
use crate::server;
use actix_web::http::StatusCode;
use futures::{future, Future};

impl Client {
    pub fn auth_login(
        &self,
        email: String,
        password: String,
    ) -> impl Future<Item = server::route::auth::LoginResponse, Error = ClientError> {
        let body = server::route::auth::LoginBody { email, password };

        self.post("/v1/auth/login")
            .send_json(&body)
            .map_err(|_err| ClientError::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => future::ok(res),
                _ => future::err(ClientError::Unwrap),
            })
            .and_then(|mut res| {
                res.json::<server::route::auth::LoginResponse>()
                    .map_err(|_err| ClientError::Unwrap)
            })
    }
}
