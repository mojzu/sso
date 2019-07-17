//! # Clients
//! Server clients.
#[cfg(feature = "async_client")]
mod async_impl;
#[cfg(feature = "sync_client")]
mod sync_impl;

#[cfg(feature = "async_client")]
pub use crate::client::async_impl::AsyncClient;
#[cfg(feature = "sync_client")]
pub use crate::client::sync_impl::SyncClient;
use crate::crate_user_agent;
use serde::ser::Serialize;
use std::error::Error as StdError;
use url::Url;

// TODO(refactor): Finish client methods.

#[derive(Debug, Fail, PartialEq)]
pub enum RequestError {
    /// Bad request error.
    #[fail(display = "RequestError::BadRequest")]
    BadRequest,
    /// Forbidden error.
    #[fail(display = "RequestError::Forbidden")]
    Forbidden,
}

/// Client errors.
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
    SerdeUrlencodedSer(#[fail(cause)] serde_urlencoded::ser::Error),
}

impl Error {
    pub fn url(err: &StdError) -> Error {
        Error::Url(err.description().into())
    }
}

/// Client options.
#[derive(Debug, Clone)]
pub struct ClientOptions {
    url: Url,
    user_agent: String,
    authorisation: String,
}

impl ClientOptions {
    pub fn new<T: Into<String>>(url: &str, authorisation: T) -> Result<Self, Error> {
        let url = Url::parse(url).map_err(|err| Error::url(&err))?;
        Ok(ClientOptions {
            url,
            user_agent: crate_user_agent(),
            authorisation: authorisation.into(),
        })
    }

    pub fn set_user_agent<T: Into<String>>(&mut self, user_agent: T) {
        self.user_agent = user_agent.into();
    }

    pub fn set_authorisation<T: Into<String>>(&mut self, authorisation: T) {
        self.authorisation = authorisation.into();
    }

    pub fn url_path(&self, path: &str) -> Result<Url, Error> {
        self.url.join(path).map_err(|err| Error::url(&err))
    }

    pub fn url_path_query<T: Serialize>(&self, path: &str, query: T) -> Result<Url, Error> {
        let mut url = self.url_path(path)?;
        let query = serde_urlencoded::to_string(query).map_err(Error::SerdeUrlencodedSer)?;
        url.set_query(Some(&query));
        Ok(url)
    }

    /// Split value of `Authorization` HTTP header into a type and value, where format is `TYPE VALUE`.
    pub fn split_authorisation(type_value: String) -> Result<(String, String), Error> {
        let mut type_value = type_value.split_whitespace();
        let type_ = type_value.next();
        if type_.is_none() {
            return Err(Error::InvalidKeyOrToken);
        }
        let type_: String = type_.unwrap().into();

        let value = type_value.next();
        if value.is_none() {
            return Err(Error::InvalidKeyOrToken);
        }
        let value: String = value.unwrap().into();
        Ok((type_, value))
    }
}

/// Client trait.
pub trait Client {
    fn new(options: ClientOptions) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::api::ServiceListQuery;

    #[test]
    fn builds_url_from_path_and_query() {
        let options = ClientOptions::new("http://localhost:9000", "authorisation-key").unwrap();
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
}
