use crate::client::async_impl::AsyncClient;
use crate::client::Error;
use crate::server::route::key::{CreateBody, CreateResponse};
use actix_web::http::StatusCode;
use futures::{future, Future};

impl AsyncClient {
    pub fn key_create(
        &self,
        name: &str,
        service_id: Option<i64>,
        user_id: Option<i64>,
    ) -> impl Future<Item = CreateResponse, Error = Error> {
        let body = CreateBody {
            name: name.to_owned(),
            service_id,
            user_id,
        };

        self.post("/v1/key")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => future::ok(res),
                _ => future::err(Error::Unwrap),
            })
            .and_then(|mut res| res.json::<CreateResponse>().map_err(|_err| Error::Unwrap))
    }
}
