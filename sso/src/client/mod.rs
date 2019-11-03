#[cfg(feature = "async_client")]
mod async_impl;
#[cfg(feature = "sync_client")]
mod sync_impl;

#[cfg(feature = "async_client")]
pub use crate::client::async_impl::*;
#[cfg(feature = "sync_client")]
pub use crate::client::sync_impl::*;

use crate::{DriverError, DriverResult, HEADER_USER_AUTHORISATION_NAME};
use http::{header, HeaderMap};
use reqwest::{
    r#async::RequestBuilder as AsyncRequestBuilder, RequestBuilder as SyncRequestBuilder,
};
use serde::ser::Serialize;
use std::{fs::File, io::Read};
use url::Url;

/// Client request options.
#[derive(Debug, Clone)]
pub struct ClientRequestOptions {
    authorisation: String,
    forwarded: String,
    user_authorisation: Option<String>,
}

impl ClientRequestOptions {
    /// Create client request options.
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

    /// Set headers on asynchronous request builder.
    pub fn request_headers(&self, req: AsyncRequestBuilder) -> AsyncRequestBuilder {
        let req = req
            .header(header::AUTHORIZATION, &self.authorisation)
            .header(header::FORWARDED, &self.forwarded);
        if let Some(user_authorisation) = &self.user_authorisation {
            req.header(HEADER_USER_AUTHORISATION_NAME, user_authorisation)
        } else {
            req
        }
    }

    /// Set headers on synchronous request builder.
    pub fn request_headers_sync(&self, req: SyncRequestBuilder) -> SyncRequestBuilder {
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

/// Client options.
#[derive(Debug, Clone)]
pub struct ClientOptions {
    user_agent: String,
    crt_pem: Option<Vec<u8>>,
    client_pem: Option<Vec<u8>>,
    request: ClientRequestOptions,
}

impl ClientOptions {
    /// Create new options.
    /// Reads CA certificate PEM file into buffer if provided.
    /// Reads client key PEM file into buffer if provided.
    pub fn new<UA>(
        user_agent: UA,
        crt_pem: Option<&str>,
        client_pem: Option<&str>,
        request: ClientRequestOptions,
    ) -> DriverResult<Self>
    where
        UA: Into<String>,
    {
        let mut options = Self {
            user_agent: user_agent.into(),
            crt_pem: None,
            client_pem: None,
            request,
        };

        if let Some(crt_pem) = crt_pem {
            let mut buf = Vec::new();
            File::open(crt_pem)
                .map_err(DriverError::StdIo)?
                .read_to_end(&mut buf)
                .map_err(DriverError::StdIo)?;
            options.crt_pem = Some(buf);
        }
        if let Some(client_pem) = client_pem {
            let mut buf = Vec::new();
            File::open(client_pem)
                .map_err(DriverError::StdIo)?
                .read_to_end(&mut buf)
                .map_err(DriverError::StdIo)?;
            options.client_pem = Some(buf);
        }

        Ok(options)
    }

    /// Default header map for Reqwest client builder.
    pub fn default_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::USER_AGENT, self.user_agent.parse().unwrap());
        headers
    }

    /// Returns reference to root PEM certificate bytes.
    pub fn crt_pem(&self) -> Option<&Vec<u8>> {
        self.crt_pem.as_ref()
    }

    /// Returns reference to client PEM certificate bytes.
    pub fn client_pem(&self) -> Option<&Vec<u8>> {
        self.client_pem.as_ref()
    }

    /// Returns request options.
    pub fn request(&self) -> &ClientRequestOptions {
        &self.request
    }

    /// Replace request options.
    pub fn set_request(mut self, request: ClientRequestOptions) -> Self {
        self.request = request;
        self
    }

    /// Build and return Url.
    pub fn url(url: &str, route: &str) -> DriverResult<Url> {
        let u = Url::parse(url).map_err(DriverError::UrlParse)?;
        u.join(route).map_err(DriverError::UrlParse)
    }

    /// Build and return Url with serialised query parameters.
    pub fn url_query<T: Serialize>(url: &str, route: &str, query: T) -> DriverResult<Url> {
        let mut url = Self::url(url, route)?;
        let query = serde_qs::to_string(&query).map_err::<DriverError, _>(Into::into)?;
        url.set_query(Some(&query));
        Ok(url)
    }

    /// Split value of `Authorization` HTTP header into a type and value, where format is `VALUE` or `TYPE VALUE`.
    /// For example `abc123def456`, `key abc123def456` and `token abc123def456`.
    /// Without a type `key` is assumed and returned.
    pub fn authorisation_type(type_value: String) -> DriverResult<(String, String)> {
        let mut type_value = type_value.split_whitespace();
        let type_ = type_value.next();
        let type_: String = type_
            .ok_or_else(|| DriverError::AuthenticateTypeNotFound)?
            .into();

        let value = type_value.next();
        if let Some(value) = value {
            Ok((type_, value.into()))
        } else {
            Ok(("key".to_owned(), type_))
        }
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
        let url = ClientOptions::url_query("http://localhost:9000", "/v1/service/", query).unwrap();
        assert_eq!(
            url.to_string(),
            "http://localhost:9000/v1/service/?gt=00000000-0000-0000-0000-000000000000&limit=10"
        );
    }

    #[test]
    fn splits_authorisation_type_none() {
        let (type_, value) = ClientOptions::authorisation_type("abcdefg".to_owned()).unwrap();
        assert_eq!(type_, "key");
        assert_eq!(value, "abcdefg");
    }

    #[test]
    fn splits_authorisation_type_key() {
        let (type_, value) = ClientOptions::authorisation_type("key abcdefg".to_owned()).unwrap();
        assert_eq!(type_, "key");
        assert_eq!(value, "abcdefg");
    }

    #[test]
    fn splits_authorisation_type_token() {
        let (type_, value) = ClientOptions::authorisation_type("token abcdefg".to_owned()).unwrap();
        assert_eq!(type_, "token");
        assert_eq!(value, "abcdefg");
    }
}
