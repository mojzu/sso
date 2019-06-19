mod async_impl;
mod sync_impl;

pub use crate::client::async_impl::AsyncClient;
pub use crate::client::sync_impl::SyncClient;
use crate::crate_user_agent;
use serde::ser::Serialize;
use std::error::Error as StdError;
use url::Url;

// TODO(feature): Client methods.

/// Client errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// TODO(refactor): Error type improvements.
    #[fail(display = "ClientError::Unwrap")]
    Unwrap,
    /// Url error wrapper.
    #[fail(display = "ClientError::Url {}", _0)]
    Url(String),
    /// Serde URL encoded serialise error wrapper.
    #[fail(display = "ClientError::SerdeUrlencodedSer {}", _0)]
    SerdeUrlencodedSer(#[fail(cause)] serde_urlencoded::ser::Error),
}

impl Error {
    pub fn url(err: &StdError) -> Error {
        Error::Url(err.description().to_owned())
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
    pub fn new(url: &str, authorisation: &str) -> Result<Self, Error> {
        let url = Url::parse(url).map_err(|err| Error::url(&err))?;
        Ok(ClientOptions {
            url,
            user_agent: crate_user_agent(),
            authorisation: authorisation.to_owned(),
        })
    }

    pub fn set_user_agent(&mut self, user_agent: &str) {
        self.user_agent = user_agent.to_owned();
    }

    pub fn set_authorisation(&mut self, authorisation: &str) {
        self.authorisation = authorisation.to_owned();
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
}

/// Client trait.
pub trait Client {
    fn new(options: ClientOptions) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::route::service::ListQuery;

    #[test]
    fn adds_serialised_query_to_url() {
        let options = ClientOptions::new("http://localhost:9000", "authorisation-key").unwrap();
        let query = ListQuery {
            gt: Some(0),
            lt: None,
            limit: Some(10),
        };
        let url = options.url_path_query("/v1/service/", &query).unwrap();
        assert_eq!(
            url.to_string(),
            "http://localhost:9000/v1/service/?gt=0&limit=10"
        );
    }
}
