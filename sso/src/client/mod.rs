mod actor;
#[cfg(feature = "async_client")]
mod async_impl;
pub mod client_msg;
mod error;
#[cfg(feature = "sync_client")]
mod sync_impl;

#[cfg(feature = "async_client")]
pub use crate::client::async_impl::*;
#[cfg(feature = "sync_client")]
pub use crate::client::sync_impl::*;
pub use crate::client::{actor::*, error::*};

use crate::{DriverError, DriverResult, HEADER_USER_AUTHORISATION_NAME};
use http::{header, HeaderMap};
use reqwest::{
    r#async::{Client as AsyncClient, RequestBuilder as AsyncRequestBuilder},
    RequestBuilder, Response,
};
use serde::{de::DeserializeOwned, ser::Serialize};
use url::Url;

/// Client options.
#[derive(Debug, Clone)]
pub struct ClientOptions {
    authorisation: String,
    forwarded: String,
    user_authorisation: Option<String>,
}

impl ClientOptions {
    /// Create new client options.
    pub fn new<A>(authorisation: A) -> Self
    where
        A: Into<String>,
    {
        Self {
            authorisation: authorisation.into(),
            forwarded: "unknown".to_owned(),
            user_authorisation: None,
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
        let req = req
            .header(header::AUTHORIZATION, &self.authorisation)
            .header(header::FORWARDED, &self.forwarded);
        if let Some(user_authorisation) = &self.user_authorisation {
            req.header(HEADER_USER_AUTHORISATION_NAME, user_authorisation)
        } else {
            req
        }
    }
}

/// Client functions.
#[derive(Debug)]
pub struct Client;

impl Client {
    pub fn build_client(user_agent: &str) -> DriverResult<AsyncClient> {
        let mut headers = HeaderMap::new();
        headers.insert(header::USER_AGENT, user_agent.parse().unwrap());
        AsyncClient::builder()
            .use_rustls_tls()
            .default_headers(headers)
            .build()
            .map_err(DriverError::Reqwest)
    }

    /// Default user agent constructed from crate name and version.
    pub fn default_user_agent() -> String {
        format!("{}/{}", crate_name!(), crate_version!())
    }

    /// Build and return Url.
    pub fn url(url: &str, route: &str) -> ClientResult<Url> {
        let u = Url::parse(url).map_err(|err| ClientError::url(&err))?;
        u.join(route).map_err(|err| ClientError::url(&err))
    }

    /// Build and return Url with serialised query parameters.
    pub fn url_query<T: Serialize>(url: &str, route: &str, query: T) -> ClientResult<Url> {
        let mut url = Client::url(url, route)?;
        let query = serde_qs::to_string(&query)
            .map_err(DriverError::serde_qs)
            .map_err(ClientError::driver)?;
        url.set_query(Some(&query));
        Ok(url)
    }

    /// Split value of `Authorization` HTTP header into a type and value, where format is `VALUE` or `TYPE VALUE`.
    /// For example `abc123def456`, `key abc123def456` and `token abc123def456`.
    /// Without a type `key` is assumed and returned.
    pub fn authorisation_type(type_value: String) -> ClientResult<(String, String)> {
        let mut type_value = type_value.split_whitespace();
        let type_ = type_value.next();
        let type_: String = type_.ok_or_else(|| ClientError::Unauthorised)?.into();

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
    pub fn result_json<T: DeserializeOwned>(res: ClientResult<String>) -> ClientResult<T> {
        let text = res?;
        serde_json::from_str::<T>(&text).map_err(Into::into)
    }

    /// Return response empty.
    pub fn result_empty(res: ClientResult<String>) -> ClientResult<()> {
        res?;
        Ok(())
    }

    /// Deserialise response body into type.
    pub fn response_json<T: DeserializeOwned>(res: Response) -> ClientResult<T> {
        res.error_for_status()
            .map_err(Into::into)
            .and_then(|mut res| res.json::<T>())
            .map_err(Into::into)
    }

    /// Return response body text.
    pub fn response_text(res: Response) -> ClientResult<String> {
        res.error_for_status()
            .map_err(Into::into)
            .and_then(|mut res| res.text())
            .map_err(Into::into)
    }

    /// Return response empty.
    pub fn response_empty(res: Response) -> ClientResult<()> {
        res.error_for_status().map_err(Into::into).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ServiceListRequestBuilder;
    use uuid::Uuid;

    #[test]
    fn builds_url_from_route_and_request() {
        let query = ServiceListRequestBuilder::default()
            .gt(Some(Uuid::nil()))
            .limit(Some(10))
            .build()
            .unwrap();
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
