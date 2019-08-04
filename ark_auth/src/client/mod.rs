//! # Server Clients
#[cfg(feature = "async_client")]
mod async_impl;
#[cfg(feature = "sync_client")]
mod sync_impl;

#[cfg(feature = "async_client")]
pub use crate::client::async_impl::AsyncClient;
#[cfg(feature = "sync_client")]
pub use crate::client::sync_impl::SyncClient;
use http::header;
use reqwest::header::HeaderMap;
use reqwest::Error as ReqwestError;
use serde::ser::Serialize;
use serde_urlencoded::ser::Error as SerdeUrlencodedSerError;
use std::error::Error as StdError;
use std::fs::File;
use std::io::Error as StdIoError;
use std::io::Read;
use url::Url;

/// ## Request Errors
#[derive(Debug, Fail, PartialEq)]
pub enum RequestError {
    /// Bad request error.
    #[fail(display = "RequestError::BadRequest")]
    BadRequest,
    /// Forbidden error.
    #[fail(display = "RequestError::Forbidden")]
    Forbidden,
    /// Not found error.
    #[fail(display = "RequestError::NotFound")]
    NotFound,
}

/// ## Client Errors
#[derive(Debug, Fail, PartialEq)]
pub enum Error {
    /// Client error.
    #[fail(display = "ClientError::Client {}", _0)]
    Client(String),
    /// Invalid key or token header value.
    #[fail(display = "ClientError::Client")]
    InvalidKeyOrToken,
    /// Request error.
    #[fail(display = "ClientError::Request {}", _0)]
    Request(RequestError),
    /// Response error.
    #[fail(display = "ClientError::Response")]
    Response,
    /// Url error wrapper.
    #[fail(display = "ClientError::Url {}", _0)]
    Url(String),
    /// Serde URL encoded serialise error wrapper.
    #[fail(display = "ClientError::SerdeUrlencodedSer {}", _0)]
    SerdeUrlencodedSer(#[fail(cause)] SerdeUrlencodedSerError),
    /// Standard IO error wrapper.
    #[fail(display = "ClientError::StdIo {}", _0)]
    StdIo(String),
}

impl Error {
    fn url(err: &StdError) -> Error {
        Error::Url(err.description().into())
    }

    fn stdio(err: &StdIoError) -> Error {
        Error::StdIo(err.description().into())
    }
}

impl From<ReqwestError> for Error {
    fn from(err: ReqwestError) -> Error {
        Error::Client(err.description().to_owned())
    }
}

/// ## Client Options
#[derive(Debug, Clone)]
pub struct ClientOptions {
    url: Url,
    user_agent: String,
    authorisation: String,
    forwarded: Option<String>,
    crt_pem: Option<Vec<u8>>,
    client_pem: Option<Vec<u8>>,
}

impl ClientOptions {
    /// Create new client options.
    /// Reads CA certificate PEM file into buffer if provided.
    /// Reads client key PEM file into buffer if provided.
    pub fn new<T1: Into<String>>(
        url: &str,
        authorisation: T1,
        forwarded: Option<String>,
        crt_pem: Option<String>,
        client_pem: Option<String>,
    ) -> Result<Self, Error> {
        let url = Url::parse(url).map_err(|err| Error::url(&err))?;
        let mut options = ClientOptions {
            url,
            user_agent: ClientOptions::default_user_agent(),
            authorisation: authorisation.into(),
            forwarded,
            crt_pem: None,
            client_pem: None,
        };

        if let Some(crt_pem) = crt_pem {
            let mut buf = Vec::new();
            File::open(&crt_pem)
                .map_err(|err| Error::stdio(&err))?
                .read_to_end(&mut buf)
                .map_err(|err| Error::stdio(&err))?;
            options.crt_pem = Some(buf);
        }
        if let Some(client_pem) = client_pem {
            let mut buf = Vec::new();
            File::open(&client_pem)
                .map_err(|err| Error::stdio(&err))?
                .read_to_end(&mut buf)
                .map_err(|err| Error::stdio(&err))?;
            options.client_pem = Some(buf);
        }

        Ok(options)
    }

    /// Clone client options with forwarded value.
    pub fn with_forwarded<T1: Into<String>>(&self, forwarded: T1) -> Self {
        // TODO(refactor): Appended user agent here.
        let mut options = self.clone();
        options.forwarded = Some(forwarded.into());
        options
    }

    /// Build URL from client options and path.
    pub fn url_path(&self, path: &str) -> Result<Url, Error> {
        self.url.join(path).map_err(|err| Error::url(&err))
    }

    /// Build URL from client options and path with serialised query parameters.
    pub fn url_path_query<T: Serialize>(&self, path: &str, query: T) -> Result<Url, Error> {
        let mut url = self.url_path(path)?;
        let query = serde_urlencoded::to_string(query).map_err(Error::SerdeUrlencodedSer)?;
        url.set_query(Some(&query));
        Ok(url)
    }

    /// Get reference to user agent header value.
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Get reference to authorisation header value.
    pub fn authorisation(&self) -> &str {
        &self.authorisation
    }

    /// Get reference to forwarded header value.
    pub fn forwarded(&self) -> Option<&str> {
        self.forwarded.as_ref().map(|x| &**x)
    }

    /// Default header map for Reqwest client builder.
    pub fn default_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(header::USER_AGENT, self.user_agent().parse().unwrap());
        headers.insert(header::AUTHORIZATION, self.authorisation().parse().unwrap());
        if let Some(forwarded) = self.forwarded() {
            headers.insert(header::FORWARDED, forwarded.parse().unwrap());
        }
        headers
    }

    /// Default user agent constructed from crate name and version.
    pub fn default_user_agent() -> String {
        format!("{}/{}", crate_name!(), crate_version!())
    }

    /// Split value of `Authorization` HTTP header into a type and value, where format is `TYPE VALUE`.
    /// For example `key abc123def456` and `token abc123def456`.
    pub fn split_authorisation(type_value: String) -> Result<(String, String), Error> {
        let mut type_value = type_value.split_whitespace();
        let type_ = type_value.next();
        let type_: String = type_.ok_or_else(|| Error::InvalidKeyOrToken)?.into();

        let value = type_value.next();
        let value: String = value.ok_or_else(|| Error::InvalidKeyOrToken)?.into();
        Ok((type_, value))
    }
}

/// ## Client Trait
/// Options are shared between synchronous and asynchronous clients.
pub trait Client {
    /// Create a new client with options.
    fn new(options: ClientOptions) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::api::ServiceListQuery;

    #[test]
    fn builds_url_from_path_and_query() {
        let options = ClientOptions::new(
            "http://localhost:9000",
            "authorisation-key",
            None,
            None,
            None,
        )
        .unwrap();
        let query = ServiceListQuery {
            gt: Some("".to_owned()),
            lt: None,
            limit: Some("10".to_owned()),
        };
        let url = options.url_path_query("/v1/service/", &query).unwrap();
        assert_eq!(
            url.to_string(),
            "http://localhost:9000/v1/service/?gt=&limit=10"
        );
    }

    #[test]
    fn splits_authorisation_key() {
        let (type_, value) = ClientOptions::split_authorisation("key abcdefg".to_owned()).unwrap();
        assert_eq!(type_, "key");
        assert_eq!(value, "abcdefg");
    }

    #[test]
    fn splits_authorisation_token() {
        let (type_, value) =
            ClientOptions::split_authorisation("token abcdefg".to_owned()).unwrap();
        assert_eq!(type_, "token");
        assert_eq!(value, "abcdefg");
    }
}
