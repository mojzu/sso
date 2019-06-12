use crate::client::{Client, ClientError};
use crate::server;
use actix_web::http::StatusCode;
use futures::{future, Future};

impl Client {
    pub fn user_create(
        &self,
        name: String,
        email: String,
        active: bool,
        password: Option<String>,
    ) -> impl Future<Item = server::route::user::CreateResponse, Error = ClientError> {
        let body = server::route::user::CreateBody {
            name,
            email,
            active,
            password,
        };

        self.post("/v1/user")
            .send_json(&body)
            .map_err(|_err| ClientError::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => future::ok(res),
                _ => future::err(ClientError::Unwrap),
            })
            .and_then(|mut res| {
                res.json::<server::route::user::CreateResponse>()
                    .map_err(|_err| ClientError::Unwrap)
            })
    }
}
