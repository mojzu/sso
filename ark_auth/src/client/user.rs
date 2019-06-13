use crate::client::{Client, ClientError};
use crate::server::route::user::{CreateBody, CreateResponse};
use actix_web::http::StatusCode;
use futures::{future, Future};

impl Client {
    pub fn user_create(
        &self,
        name: &str,
        email: &str,
        active: bool,
        password: Option<&str>,
    ) -> impl Future<Item = CreateResponse, Error = ClientError> {
        let body = CreateBody {
            name: name.to_owned(),
            email: email.to_owned(),
            active,
            password: password.map(String::from),
        };

        self.post("/v1/user")
            .send_json(&body)
            .map_err(|_err| ClientError::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => future::ok(res),
                _ => future::err(ClientError::Unwrap),
            })
            .and_then(|mut res| {
                res.json::<CreateResponse>()
                    .map_err(|_err| ClientError::Unwrap)
            })
    }
}
