use crate::{Client, ClientError, ClientResult};
use actix::prelude::*;
use http::{header, HeaderMap};
use reqwest::r#async::{Client as ReqwestClient, ClientBuilder};
use std::{fmt, fs::File, io::Read};

/// Client actor options.
#[derive(Debug, Clone)]
pub struct ClientActorOptions {
    user_agent: String,
    crt_pem: Option<Vec<u8>>,
    client_pem: Option<Vec<u8>>,
}

impl ClientActorOptions {
    /// Create new options.
    /// Reads CA certificate PEM file into buffer if provided.
    /// Reads client key PEM file into buffer if provided.
    pub fn new<T1>(
        user_agent: T1,
        crt_pem: Option<&str>,
        client_pem: Option<&str>,
    ) -> ClientResult<Self>
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
                .map_err(|e| ClientError::stdio(&e))?
                .read_to_end(&mut buf)
                .map_err(|e| ClientError::stdio(&e))?;
            options.crt_pem = Some(buf);
        }
        if let Some(client_pem) = client_pem {
            let mut buf = Vec::new();
            File::open(client_pem)
                .map_err(|e| ClientError::stdio(&e))?
                .read_to_end(&mut buf)
                .map_err(|e| ClientError::stdio(&e))?;
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

impl Default for ClientActorOptions {
    fn default() -> Self {
        ClientActorOptions::new(Client::default_user_agent(), None, None).unwrap()
    }
}

/// Client Actor.
pub struct ClientActor {
    client: ReqwestClient,
}

impl fmt::Debug for ClientActor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientActor {{ client }}")
    }
}

impl ClientActor {
    /// Get client reference.
    pub fn client(&self) -> &ReqwestClient {
        &self.client
    }

    /// Start client actor with options.
    /// Reqwest advises not to recreate clients, and clients cannot be sent across threads.
    /// The primary use case for asynchronous client is in actix-web routes, which may
    /// run across many threads, so to avoid creating a client for each request, the client
    /// can be run in an actor thread and used by passing messages.
    pub fn start(options: ClientActorOptions) -> Addr<Self> {
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

impl Supervised for ClientActor {}

impl Actor for ClientActor {
    type Context = Context<Self>;
}

/// Client actor message request trait.
pub trait ClientActorRequest {
    /// Set authorisation header on request.
    fn authorisation<T1: Into<String>>(self, authorisation: T1) -> Self;
    /// Set forwarded header on request.
    fn forwarded<T1: Into<String>>(self, forwarded: T1) -> Self;
}
