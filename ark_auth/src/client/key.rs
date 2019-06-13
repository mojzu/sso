use crate::client::{Client, ClientError};
use crate::server::route::key::{CreateBody, CreateResponse};
use actix_web::http::StatusCode;
use futures::{future, Future};

impl Client {
    pub fn key_create(
        &self,
        name: &str,
        service_id: Option<i64>,
        user_id: Option<i64>,
    ) -> impl Future<Item = CreateResponse, Error = ClientError> {
        let body = CreateBody {
            name: name.to_owned(),
            service_id,
            user_id,
        };

        self.post("/v1/key")
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
