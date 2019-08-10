//! # Server Clients
#[cfg(feature = "async_client")]
mod async_impl;
mod error;
#[cfg(feature = "sync_client")]
mod sync_impl;

#[cfg(feature = "async_client")]
pub use crate::client::async_impl::AsyncClient;
#[cfg(feature = "sync_client")]
pub use crate::client::sync_impl::SyncClient;
pub use error::Error as ClientError;

use crate::client::error::Error;
use actix::prelude::*;
use http::{header, HeaderMap};
use reqwest::r#async::{Client as ReqwestClient, ClientBuilder};
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::fs::File;
use std::io::Read;
use url::Url;

/// Default user agent constructed from crate name and version.
pub fn default_user_agent() -> String {
    format!("{}/{}", crate_name!(), crate_version!())
}

/// ## Client Executor Options
#[derive(Debug, Clone)]
pub struct ClientExecutorOptions {
    user_agent: String,
    crt_pem: Option<Vec<u8>>,
    client_pem: Option<Vec<u8>>,
}

impl ClientExecutorOptions {
    /// Create new options.
    /// Reads CA certificate PEM file into buffer if provided.
    /// Reads client key PEM file into buffer if provided.
    pub fn new<T1>(
        user_agent: T1,
        crt_pem: Option<&str>,
        client_pem: Option<&str>,
    ) -> Result<Self, Error>
    where
        T1: Into<String>,
    {
        let mut options = Self {
            user_agent: user_agent.into(),
            crt_pem: None,
            client_pem: None,
        };

        if let Some(crt_pem) = crt_pem {
            let mut buf = Vec::new();
            File::open(crt_pem)
                .map_err(Into::into)?
                .read_to_end(&mut buf)
                .map_err(Into::into)?;
            options.crt_pem = Some(buf);
        }
        if let Some(client_pem) = client_pem {
            let mut buf = Vec::new();
            File::open(client_pem)
                .map_err(Into::into)?
                .read_to_end(&mut buf)
                .map_err(Into::into)?;
            options.client_pem = Some(buf);
        }

        Ok(options)
    }

    /// Default header map for Reqwest client builder.
    pub fn default_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::USER_AGENT, self.user_agent.parse().unwrap());
        // TODO(refactor): Improved forwarded header format.
        // <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Forwarded>
        headers
    }
}

/// ## Client Executor
/// Reqwest advises not to recreate clients, and clients cannot be sent across threads.
/// The primary use case for asynchronous client is in actix-web routes, which may
/// run across many threads, so to avoid creating a client for each request, the client
/// can be run in an actor thread and used by passing messages.
pub struct ClientExecutor {
    client: ReqwestClient,
}

impl ClientExecutor {
    /// Start client actor with options.
    pub fn start(options: ClientExecutorOptions) -> Addr<Self> {
        Supervisor::start(move |_| {
            let headers = options.default_headers();
            let builder = ClientBuilder::new()
                .use_rustls_tls()
                .default_headers(headers);
            let builder = match &options.crt_pem {
                Some(buf) => {
                    let crt_pem = reqwest::Certificate::from_pem(buf).unwrap();
                    builder.add_root_certificate(crt_pem)
                }
                None => builder,
            };
            let builder = match &options.client_pem {
                Some(buf) => {
                    let client_pem = reqwest::Identity::from_pem(buf).unwrap();
                    builder.identity(client_pem)
                }
                None => builder,
            };
            let client = builder.build().unwrap();

            Self { client }
        })
    }
}

impl Supervised for ClientExecutor {}

impl Actor for ClientExecutor {
    type Context = Context<Self>;
}

/// ## Asynchronous Client GET Request
#[derive(Debug, Serialize, Deserialize)]
pub struct Get {
    url: String,
    route: String,
    content_type: String,
    authorisation: Option<String>,
    forwarded: Option<String>,
}

impl Get {
    /// Create new GET text request.
    pub fn text<T1, T2>(url: T1, route: T2) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Self {
            url: url.into(),
            route: route.into(),
            content_type: "text/plain".to_owned(),
            authorisation: None,
            forwarded: None,
        }
    }

    /// Create new GET JSON request.
    pub fn json<T1, T2>(url: T1, route: T2) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Self {
            url: url.into(),
            route: route.into(),
            content_type: "application/json".to_owned(),
            authorisation: None,
            forwarded: None,
        }
    }

    /// Set authorisation header on GET request.
    pub fn authorisation<T1: Into<String>>(mut self, authorisation: T1) -> Self {
        self.authorisation = Some(authorisation.into());
        self
    }

    /// Set forwarded header on GET request.
    pub fn forwarded<T1: Into<String>>(mut self, forwarded: T1) -> Self {
        self.forwarded = Some(forwarded.into());
        self
    }
}

impl Message for Get {
    type Result = Result<String, Error>;
}

impl Handler<Get> for ClientExecutor {
    type Result = ResponseActFuture<Self, String, Error>;

    fn handle(&mut self, msg: Get, _ctx: &mut Context<Self>) -> Self::Result {
        let url = Client::url(&msg.url, &msg.route).unwrap();
        let req = self
            .client
            .get(url)
            .header(header::CONTENT_TYPE, &msg.content_type);
        let req = if let Some(authorisation) = &msg.authorisation {
            req.header(header::AUTHORIZATION, authorisation)
        } else {
            req
        };
        let req = if let Some(forwarded) = &msg.forwarded {
            req.header(header::FORWARDED, forwarded)
        } else {
            req
        };

        let res = req
            .send()
            .map_err(Into::into)
            .and_then(|res| res.error_for_status().map_err(Into::into))
            .and_then(|mut res| res.text().map_err(Into::into));

        let wrapped = actix::fut::wrap_future(res);
        Box::new(wrapped)
    }
}

/// ## Client Options
#[derive(Debug, Clone)]
pub struct ClientOptions {
    url: String,
    authorisation: String,
}

impl ClientOptions {
    /// Create new client options.
    pub fn new<T1, T2>(url: T1, authorisation: T2) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Self {
            url: url.into(),
            authorisation: authorisation.into(),
        }
    }

    /// Returns url reference.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns authorisation reference.
    pub fn authorisation(&self) -> &str {
        &self.authorisation
    }
}

/// ## Client Utilities
pub struct Client;

impl Client {
    /// Build and return Url.
    pub fn url(url: &str, route: &str) -> Result<Url, Error> {
        let u = Url::parse(url).map_err(|err| Error::url(&err))?;
        u.join(route).map_err(|err| Error::url(&err))
    }

    /// Build and return Url with serialised query parameters.
    pub fn url_query<T: Serialize>(url: &str, route: &str, query: T) -> Result<Url, Error> {
        let mut url = Client::url(url, route)?;
        let query = serde_urlencoded::to_string(query).map_err(Error::SerdeUrlencodedSer)?;
        url.set_query(Some(&query));
        Ok(url)
    }

    /// Split value of `Authorization` HTTP header into a type and value, where format is `VALUE` or `TYPE VALUE`.
    /// For example `abc123def456`, `key abc123def456` and `token abc123def456`.
    /// Without a type `key` is assumed and returned.
    pub fn authorisation_type(type_value: String) -> Result<(String, String), Error> {
        let mut type_value = type_value.split_whitespace();
        let type_ = type_value.next();
        let type_: String = type_.ok_or_else(|| Error::Forbidden)?.into();

        let value = type_value.next();
        if let Some(value) = value {
            Ok((type_, value.into()))
        } else {
            Ok(("key".to_owned(), type_))
        }
    }

    /// Deserialise response text into type.
    pub fn result_json<T: DeserializeOwned>(res: Result<String, Error>) -> Result<T, Error> {
        let text = res?;
        serde_json::from_str::<T>(&text).map_err(Into::into)
    }

    /// Deserialise response body into type.
    pub fn response_json<T: DeserializeOwned>(res: Response) -> Result<T, Error> {
        let res = res.error_for_status().map_err(Into::into)?;
        res.json::<T>().map_err(Into::into)
    }

    /// Return response body text.
    pub fn response_text(res: Response) -> Result<String, Error> {
        let res = res.error_for_status().map_err(Into::into)?;
        res.text().map_err(Into::into)
    }

    /// Return response empty.
    pub fn response_empty(res: Response) -> Result<(), Error> {
        res.error_for_status().map_err(Into::into)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::api::ServiceListQuery;

    #[test]
    fn builds_url_from_route_and_query() {
        let query = ServiceListQuery {
            gt: Some("".to_owned()),
            lt: None,
            limit: Some("10".to_owned()),
        };
        let url = Client::url_query("http://localhost:9000", "/v1/service/", query).unwrap();
        assert_eq!(
            url.to_string(),
            "http://localhost:9000/v1/service/?gt=&limit=10"
        );
    }

    #[test]
    fn splits_authorisation_type_none() {
        let (type_, value) = Client::authorisation_type("abcdefg".to_owned()).unwrap();
        assert_eq!(type_, "key");
        assert_eq!(value, "abcdefg");
    }

    #[test]
    fn splits_authorisation_type_key() {
        let (type_, value) = Client::authorisation_type("key abcdefg".to_owned()).unwrap();
        assert_eq!(type_, "key");
        assert_eq!(value, "abcdefg");
    }

    #[test]
    fn splits_authorisation_type_token() {
        let (type_, value) = Client::authorisation_type("token abcdefg".to_owned()).unwrap();
        assert_eq!(type_, "token");
        assert_eq!(value, "abcdefg");
    }
}
