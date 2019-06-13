mod key;
mod service;
mod user;

use actix_web::http::header;
use serde::ser::Serialize;
use url::Url;

// TODO(feature): Client methods.

/// Client errors.
#[derive(Debug, Fail)]
pub enum ClientError {
    /// TODO(refactor): Error type improvements.
    #[fail(display = "ClientError::Unwrap")]
    Unwrap,
}

/// Client options.
pub struct ClientOptions {
    url: Url,
    user_agent: String,
    authorisation: String,
}

/// Client handle.
pub struct Client {
    options: ClientOptions,
    client: actix_web::client::Client,
}

impl ClientOptions {
    pub fn new(url: &str, user_agent: String, authorisation: String) -> Self {
        ClientOptions {
            url: Url::parse(url).unwrap(),
            user_agent,
            authorisation,
        }
    }
}

impl Client {
    pub fn new(options: ClientOptions) -> Self {
        let client = Client::build_client(&options);
        Client { options, client }
    }

    pub fn set_authorisation(mut self, authorisation: String) -> Self {
        self.options.authorisation = authorisation;
        self
    }

    fn build_client(options: &ClientOptions) -> actix_web::client::Client {
        actix_web::client::Client::build()
            .header(header::CONTENT_TYPE, header::ContentType::json())
            .header(header::USER_AGENT, options.user_agent.to_owned())
            .finish()
    }

    fn url_path(&self, path: &str) -> Url {
        self.options.url.join(path).unwrap()
    }

    fn get(&self, path: &str) -> actix_web::client::ClientRequest {
        let url = self.url_path(path);

        self.client
            .get(url.to_string())
            .header(header::AUTHORIZATION, self.options.authorisation.to_owned())
    }

    fn get_query<T: Serialize>(&self, path: &str, query: T) -> actix_web::client::ClientRequest {
        let mut url = self.url_path(path);
        let query = serde_urlencoded::to_string(query).unwrap();
        url.set_query(Some(&query));

        self.client
            .get(url.to_string())
            .header(header::AUTHORIZATION, self.options.authorisation.to_owned())
    }

    fn post(&self, path: &str) -> actix_web::client::ClientRequest {
        let url = self.url_path(path);

        self.client
            .post(url.to_string())
            .header(header::AUTHORIZATION, self.options.authorisation.to_owned())
    }
}
