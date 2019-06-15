mod auth;
mod key;
mod service;
mod user;

use crate::client::{Client, ClientOptions, Error};
use actix_web::http::{header, StatusCode};
use futures::{future, Future};
use serde::ser::Serialize;
use serde_json::Value;

/// Asynchronous client handle.
pub struct AsyncClient {
    pub options: ClientOptions,
    pub client: actix_web::client::Client,
}

impl AsyncClient {
    pub fn ping(&self) -> impl Future<Item = Value, Error = Error> {
        self.get("/v1/ping")
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => future::ok(res),
                _ => future::err(Error::Unwrap),
            })
            .and_then(|mut res| res.json::<Value>().map_err(|_err| Error::Unwrap))
    }

    fn build_client(options: &ClientOptions) -> actix_web::client::Client {
        actix_web::client::Client::build()
            .header(header::CONTENT_TYPE, header::ContentType::json())
            .header(header::USER_AGENT, options.user_agent.to_owned())
            .finish()
    }

    fn get(&self, path: &str) -> actix_web::client::ClientRequest {
        let url = self.options.url_path(path);
        self.client
            .get(url.to_string())
            .header(header::AUTHORIZATION, self.options.authorisation.to_owned())
    }

    fn get_query<T: Serialize>(&self, path: &str, query: T) -> actix_web::client::ClientRequest {
        let url = self.options.url_path_query(path, query);
        self.client
            .get(url.to_string())
            .header(header::AUTHORIZATION, self.options.authorisation.to_owned())
    }

    fn post(&self, path: &str) -> actix_web::client::ClientRequest {
        let url = self.options.url_path(path);
        self.client
            .post(url.to_string())
            .header(header::AUTHORIZATION, self.options.authorisation.to_owned())
    }
}

impl Client for AsyncClient {
    fn new(options: ClientOptions) -> Self {
        let client = AsyncClient::build_client(&options);
        AsyncClient { options, client }
    }
}
