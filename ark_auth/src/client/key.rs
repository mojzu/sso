use crate::client::{Client, ClientError};
use crate::server;
use actix_web::http::StatusCode;
use futures::{future, Future};

impl Client {
    pub fn key_create(
        &self,
        name: String,
        service_id: Option<i64>,
        user_id: Option<i64>,
    ) -> impl Future<Item = server::route::key::CreateResponse, Error = ClientError> {
        let body = server::route::key::CreateBody {
            name,
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
                res.json::<server::route::key::CreateResponse>()
                    .map_err(|_err| ClientError::Unwrap)
            })
    }
}
