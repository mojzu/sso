mod async_impl;
mod sync_impl;

pub use crate::client::async_impl::AsyncClient;
pub use crate::client::sync_impl::SyncClient;
use serde::ser::Serialize;
use url::Url;

// TODO(feature): Client methods.

/// Client errors.
#[derive(Debug, Fail)]
pub enum Error {
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

impl ClientOptions {
    pub fn new(url: &str, user_agent: &str, authorisation: &str) -> Self {
        ClientOptions {
            url: Url::parse(url).unwrap(),
            user_agent: user_agent.to_owned(),
            authorisation: authorisation.to_owned(),
        }
    }

    pub fn set_authorisation(&mut self, authorisation: &str) {
        self.authorisation = authorisation.to_owned();
    }

    pub fn url_path(&self, path: &str) -> Url {
        self.url.join(path).unwrap()
    }

    pub fn url_path_query<T: Serialize>(&self, path: &str, query: T) -> Url {
        let mut url = self.url_path(path);
        let query = serde_urlencoded::to_string(query).unwrap();
        url.set_query(Some(&query));
        url
    }
}

/// Client trait.
pub trait Client {
    fn new(options: ClientOptions) -> Self;
}
