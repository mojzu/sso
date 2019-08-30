//! # Server Clients
mod actor;
#[cfg(feature = "async_client")]
mod async_impl;
mod error;
#[cfg(feature = "sync_client")]
mod sync_impl;

#[cfg(feature = "async_client")]
pub use crate::client::async_impl::AsyncClient;
#[cfg(feature = "sync_client")]
pub use crate::client::sync_impl::SyncClient;
pub use crate::client::{
    actor::{
        ClientActor, ClientActorOptions, ClientActorRequest, Delete, Get, PatchJson, PostJson,
    },
    error::Error as ClientError,
};

use crate::client::error::Error;
use http::header;
use reqwest::{r#async::RequestBuilder as AsyncRequestBuilder, RequestBuilder, Response};
use serde::{de::DeserializeOwned, ser::Serialize};
use url::Url;

/// Default user agent constructed from crate name and version.
pub fn default_user_agent() -> String {
    format!("{}/{}", crate_name!(), crate_version!())
}

/// ## Client Options
#[derive(Debug, Clone)]
pub struct ClientOptions {
    authorisation: String,
    forwarded: String,
}

impl ClientOptions {
    /// Create new client options.
    pub fn new<T1: Into<String>>(authorisation: T1) -> Self {
        Self {
            authorisation: authorisation.into(),
            forwarded: "unknown".to_owned(),
        }
    }

    /// Returns authorisation reference.
    pub fn authorisation(&self) -> &str {
        &self.authorisation
    }

    /// Returns forwarded reference.
    pub fn forwarded(&self) -> &str {
        &self.forwarded
    }

    /// Set headers on synchronous request builder.
    pub fn request_headers(&self, req: RequestBuilder) -> RequestBuilder {
        req.header(header::AUTHORIZATION, &self.authorisation)
            .header(header::FORWARDED, &self.forwarded)
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

    /// Serialize value as URL encoded query parameters string.
    pub fn query_string<S: Serialize>(query: S) -> Result<String, Error> {
        serde_urlencoded::to_string(query).map_err(Error::SerdeUrlencodedSer)
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

    /// Set headers on asynchronous request builder.
    pub fn request_headers(
        req: AsyncRequestBuilder,
        authorisation: Option<String>,
        forwarded: Option<String>,
    ) -> AsyncRequestBuilder {
        let req = if let Some(authorisation) = authorisation {
            req.header(header::AUTHORIZATION, authorisation)
        } else {
            req
        };
        if let Some(forwarded) = forwarded {
            req.header(header::FORWARDED, forwarded)
        } else {
            req
        }
    }

    /// Deserialise response text into type.
    pub fn result_json<T: DeserializeOwned>(res: Result<String, Error>) -> Result<T, Error> {
        let text = res?;
        serde_json::from_str::<T>(&text).map_err(Into::into)
    }

    /// Return response empty.
    pub fn result_empty(res: Result<String, Error>) -> Result<(), Error> {
        res?;
        Ok(())
    }

    /// Deserialise response body into type.
    pub fn response_json<T: DeserializeOwned>(res: Response) -> Result<T, Error> {
        res.error_for_status()
            .map_err(Into::into)
            .and_then(|mut res| res.json::<T>())
            .map_err(Into::into)
    }

    /// Return response body text.
    pub fn response_text(res: Response) -> Result<String, Error> {
        res.error_for_status()
            .map_err(Into::into)
            .and_then(|mut res| res.text())
            .map_err(Into::into)
    }

    /// Return response empty.
    pub fn response_empty(res: Response) -> Result<(), Error> {
        res.error_for_status().map_err(Into::into).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::api::ServiceListQuery;
    use uuid::Uuid;

    #[test]
    fn builds_url_from_route_and_query() {
        let query = ServiceListQuery {
            gt: Some(Uuid::nil()),
            lt: None,
            limit: Some("10".to_owned()),
        };
        let url = Client::url_query("http://localhost:9000", "/v1/service/", query).unwrap();
        assert_eq!(
            url.to_string(),
            "http://localhost:9000/v1/service/?gt=00000000-0000-0000-0000-000000000000&limit=10"
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
