use crate::client::error::Error;
use crate::client::Client;
use actix::prelude::*;
use http::{header, HeaderMap};
use reqwest::r#async::{Client as ReqwestClient, ClientBuilder};
use serde::ser::Serialize;
use std::fs::File;
use std::io::Read;

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
                .map_err(|e| Error::stdio(&e))?
                .read_to_end(&mut buf)
                .map_err(|e| Error::stdio(&e))?;
            options.crt_pem = Some(buf);
        }
        if let Some(client_pem) = client_pem {
            let mut buf = Vec::new();
            File::open(client_pem)
                .map_err(|e| Error::stdio(&e))?
                .read_to_end(&mut buf)
                .map_err(|e| Error::stdio(&e))?;
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

    /// Returns reference to root PEM certificate bytes.
    pub fn crt_pem(&self) -> Option<&Vec<u8>> {
        self.crt_pem.as_ref()
    }

    /// Returns reference to client PEM certificate bytes.
    pub fn client_pem(&self) -> Option<&Vec<u8>> {
        self.client_pem.as_ref()
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
    query: Option<String>,
    // TODO(refactor): Headers HashMap.
    authorisation: Option<String>,
    forwarded: Option<String>,
}

impl Get {
    /// Create new GET request.
    pub fn new<T1, T2>(url: T1, route: T2) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Self {
            url: url.into(),
            route: route.into(),
            query: None,
            authorisation: None,
            forwarded: None,
        }
    }

    /// Set query string on GET request URL.
    pub fn query<S: Serialize>(mut self, query: S) -> Result<Self, Error> {
        let query = Client::query_string(query)?;
        self.query = Some(query);
        Ok(self)
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
        let req = self.client.get(url);
        let req = Client::request_headers(req, msg.authorisation, msg.forwarded);

        let res = req
            .send()
            .map_err(Into::into)
            .and_then(|res| res.error_for_status().map_err(Into::into))
            .and_then(|mut res| res.text().map_err(Into::into));

        let wrapped = actix::fut::wrap_future(res);
        Box::new(wrapped)
    }
}

/// ## Asynchronous Client POST JSON Request
pub struct PostJson<S: Serialize> {
    url: String,
    route: String,
    authorisation: Option<String>,
    forwarded: Option<String>,
    body: Option<S>,
}

impl<S: Serialize> PostJson<S> {
    /// Create new POST JSON request.
    pub fn new<T1, T2>(url: T1, route: T2, body: Option<S>) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Self {
            url: url.into(),
            route: route.into(),
            authorisation: None,
            forwarded: None,
            body,
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

impl<S: Serialize> Message for PostJson<S> {
    type Result = Result<String, Error>;
}

impl<S: Serialize> Handler<PostJson<S>> for ClientExecutor {
    type Result = ResponseActFuture<Self, String, Error>;

    fn handle(&mut self, msg: PostJson<S>, _ctx: &mut Context<Self>) -> Self::Result {
        let url = Client::url(&msg.url, &msg.route).unwrap();
        let req = self.client.post(url);
        let req = Client::request_headers(req, msg.authorisation, msg.forwarded);
        let req = if let Some(body) = msg.body {
            req.json(&body)
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

/// ## Asynchronous Client PATCH Request
pub struct PatchJson<S: Serialize> {
    url: String,
    route: String,
    authorisation: Option<String>,
    forwarded: Option<String>,
    body: Option<S>,
}

impl<S: Serialize> PatchJson<S> {
    /// Create new Serialize JSON request.
    pub fn new<T1, T2>(url: T1, route: T2, body: Option<S>) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Self {
            url: url.into(),
            route: route.into(),
            authorisation: None,
            forwarded: None,
            body,
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

impl<S: Serialize> Message for PatchJson<S> {
    type Result = Result<String, Error>;
}

impl<S: Serialize> Handler<PatchJson<S>> for ClientExecutor {
    type Result = ResponseActFuture<Self, String, Error>;

    fn handle(&mut self, msg: PatchJson<S>, _ctx: &mut Context<Self>) -> Self::Result {
        let url = Client::url(&msg.url, &msg.route).unwrap();
        let req = self.client.patch(url);
        let req = Client::request_headers(req, msg.authorisation, msg.forwarded);
        let req = if let Some(body) = msg.body {
            req.json(&body)
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

/// ## Asynchronous Client DELETE Request
#[derive(Debug, Serialize, Deserialize)]
pub struct Delete {
    url: String,
    route: String,
    authorisation: Option<String>,
    forwarded: Option<String>,
}

impl Delete {
    /// Create new DELETE request.
    pub fn new<T1, T2>(url: T1, route: T2) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Self {
            url: url.into(),
            route: route.into(),
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

impl Message for Delete {
    type Result = Result<String, Error>;
}

impl Handler<Delete> for ClientExecutor {
    type Result = ResponseActFuture<Self, String, Error>;

    fn handle(&mut self, msg: Delete, _ctx: &mut Context<Self>) -> Self::Result {
        let url = Client::url(&msg.url, &msg.route).unwrap();
        let req = self.client.delete(url);
        let req = Client::request_headers(req, msg.authorisation, msg.forwarded);

        let res = req
            .send()
            .map_err(Into::into)
            .and_then(|res| res.error_for_status().map_err(Into::into))
            .and_then(|mut res| res.text().map_err(Into::into));

        let wrapped = actix::fut::wrap_future(res);
        Box::new(wrapped)
    }
}
