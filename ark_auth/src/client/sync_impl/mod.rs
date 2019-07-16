mod audit;
mod auth;
mod key;
mod service;
mod user;

use crate::client::{Client, ClientOptions, Error, RequestError};
use crate::server::api::route;
use actix_web::http::{header, StatusCode};
use serde::ser::Serialize;
use serde_json::Value;
use std::error::Error as StdError;

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Client(err.description().to_owned())
    }
}

/// Synchronous client handle.
pub struct SyncClient {
    pub options: ClientOptions,
    pub client: reqwest::Client,
}

impl SyncClient {
    pub fn ping(&self) -> Result<Value, Error> {
        self.get(route::PING)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<Value>().map_err(Into::into))
    }

    fn build_client(options: &ClientOptions) -> reqwest::Client {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(header::USER_AGENT, options.user_agent.parse().unwrap());

        reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap()
    }

    fn get(&self, path: &str) -> reqwest::RequestBuilder {
        let url = self.options.url_path(path).unwrap();
        self.client
            .get(url)
            .header(header::AUTHORIZATION, self.options.authorisation.to_owned())
    }

    fn get_query<T: Serialize>(&self, path: &str, query: T) -> reqwest::RequestBuilder {
        let url = self.options.url_path_query(path, query).unwrap();
        self.client
            .get(url)
            .header(header::AUTHORIZATION, self.options.authorisation.to_owned())
    }

    fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let url = self.options.url_path(path).unwrap();
        self.client
            .post(url)
            .header(header::AUTHORIZATION, self.options.authorisation.to_owned())
    }

    fn post_json<T: Serialize>(&self, path: &str, body: &T) -> reqwest::RequestBuilder {
        let url = self.options.url_path(path).unwrap();
        self.client
            .post(url)
            .header(header::AUTHORIZATION, self.options.authorisation.to_owned())
            .json(&body)
    }

    fn match_status_code(response: reqwest::Response) -> Result<reqwest::Response, Error> {
        match response.status() {
            StatusCode::OK => Ok(response),
            StatusCode::BAD_REQUEST => Err(Error::Request(RequestError::BadRequest)),
            StatusCode::FORBIDDEN => Err(Error::Request(RequestError::Forbidden)),
            _ => Err(Error::Response),
        }
    }
}

impl Client for SyncClient {
    fn new(options: ClientOptions) -> Self {
        let client = SyncClient::build_client(&options);
        SyncClient { options, client }
    }
}
