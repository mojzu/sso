use crate::client::{Client, ClientError};
use crate::server;
use actix_web::http::StatusCode;
use futures::{future, Future};

impl Client {
    pub fn service_list(
        &self,
        gt: Option<i64>,
        lt: Option<i64>,
        limit: Option<i64>,
    ) -> impl Future<Item = server::route::service::ListResponse, Error = ClientError> {
        let query = server::route::service::ListQuery { gt, lt, limit };

        self.get_query("/v1/service", query)
            .send()
            .map_err(|_err| ClientError::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => future::ok(res),
                _ => future::err(ClientError::Unwrap),
            })
            .and_then(|mut res| {
                res.json::<server::route::service::ListResponse>()
                    .map_err(|_err| ClientError::Unwrap)
            })
    }

    pub fn service_create(
        &self,
        name: String,
        url: String,
    ) -> impl Future<Item = server::route::service::CreateResponse, Error = ClientError> {
        let body = server::route::service::CreateBody { name, url };

        self.post("/v1/service")
            .send_json(&body)
            .map_err(|_err| ClientError::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => future::ok(res),
                _ => future::err(ClientError::Unwrap),
            })
            .and_then(|mut res| {
                res.json::<server::route::service::CreateResponse>()
                    .map_err(|_err| ClientError::Unwrap)
            })
    }

    pub fn service_read(
        &self,
        id: i64,
    ) -> impl Future<Item = server::route::service::ReadResponse, Error = ClientError> {
        let path = format!("/v1/service/{}", id);

        self.get(&path)
            .send()
            .map_err(|_err| ClientError::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => future::ok(res),
                _ => future::err(ClientError::Unwrap),
            })
            .and_then(|mut res| {
                res.json::<server::route::service::ReadResponse>()
                    .map_err(|_err| ClientError::Unwrap)
            })
    }
}
