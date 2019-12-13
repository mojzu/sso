#[macro_use]
extern crate serde_derive;

mod error;

pub use error::{ApiError, ApiErrors};

use chrono::{DateTime, Utc};
use futures_util::future::poll_fn;
use http::Method;
use hyper::server::conn::AddrStream;
use hyper::service::make_service_fn;
use hyper::{Body, Request, Response, Server};
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

/// Type alias for future trait objects returned by `Operation`.
pub type ApiOperationFuture = dyn Future<Output = Result<Response<Body>, ApiErrors>> + Send;

/// Representation of API operation.
pub trait ApiOperation: Send + Sync {
    /// Returns HTTP method for this operation.
    fn method(&self) -> Method;
    /// Call this operation on request, returning a boxed future.
    fn call(&self, meta: &ApiRequest, req: Request<Body>) -> Box<ApiOperationFuture>;
}

/// Returns self as boxed `Operation` trait object.
pub trait ApiOperationInto: 'static + Sized + ApiOperation {
    fn into_operation(self) -> Box<dyn ApiOperation> {
        Box::new(self)
    }
}

/// Representation of API path.
pub struct ApiPath {
    /// URL path string.
    path: String,
    /// Operations supported by this path.
    operations: HashMap<Method, Box<dyn ApiOperation>>,
}

impl ApiPath {
    /// Returns new `Path` with URL path string.
    pub fn new<P: Into<String>>(path: P) -> Self {
        Self {
            path: path.into(),
            operations: HashMap::new(),
        }
    }

    /// Returns URL path string reference.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Add `Operation` trait object to `Path`.
    pub fn operation<O: ApiOperationInto>(mut self, operation: O) -> Self {
        self.operations
            .insert(operation.method(), operation.into_operation());
        self
    }

    /// Call `ApiOperation` for request HTTP method, or response with method not allowed
    /// error if no operation exists for HTTTP method. Returns a boxed future.
    pub(crate) fn call(
        &self,
        meta: &ApiRequest,
        req: Request<Body>,
    ) -> Pin<Box<ApiOperationFuture>> {
        match self.operations.get(req.method()) {
            Some(operation) => operation.call(meta, req).into(),
            None => {
                let res = ApiError::method_not_allowed(1, "apiOperationNotFound").into_response();
                let fut = async { Ok(res) };
                Box::pin(fut)
            }
        }
    }
}

/// Representation of API.
pub struct Api {
    /// Paths supported by this API.
    paths: HashMap<String, Box<ApiPath>>,
}

impl Api {
    /// Returns new `Api` with paths.
    pub fn new(paths: Vec<ApiPath>) -> Self {
        let mut h = HashMap::new();
        for path in paths {
            let p = path.path().to_owned();
            h.insert(p, Box::new(path));
        }
        Self { paths: h }
    }

    /// Bind and serve API on new [Runtime], this blocks on [Server] future.
    pub fn run(self, addr: &SocketAddr) -> Result<(), hyper::Error> {
        let api = Arc::new(self);

        let make_api = make_service_fn(move |socket: &AddrStream| {
            let request = ApiRequest::new(api.clone(), socket.remote_addr());
            async move { Ok::<_, http::Error>(request) }
        });

        let server = Server::bind(&addr).serve(make_api);

        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(server)
    }

    /// Deserialise request URL query parameters into type.
    pub fn query_parameters<'de, T>(req: &'de Request<Body>) -> Result<T, ApiErrors>
    where
        T: serde::de::Deserialize<'de>,
    {
        let i = req.uri().query().unwrap_or("");
        serde_urlencoded::from_str::<T>(i).map_err(|e| ApiErrors::new(e))
    }

    /// Deserialise JSON request body into type.
    pub async fn body<T>(body: Body) -> Result<T, ApiErrors>
    where
        T: serde::de::DeserializeOwned,
    {
        let s = hyper::body::to_bytes(body).await.unwrap().to_vec();
        serde_json::from_slice::<T>(&s).map_err(|e| ApiErrors::new(e))
    }

    /// Run a blocking closure on thread pool.
    pub fn blocking<T, E, F>(f: F) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>
    where
        F: Send + FnOnce() -> Result<T, E> + 'static,
        T: Send,
        E: Send,
    {
        let mut f = Some(f);
        let fut = async move {
            poll_fn(|_| {
                tokio_executor::threadpool::blocking(|| (f.take().unwrap())())
                    .map_err(|_| panic!("threadpool shut down"))
            })
            .await
            .unwrap()
        };
        Box::pin(fut)
    }

    /// Call `ApiPath` for request URL path, or respond with not found error if no
    /// path exists for URL path. Returns a boxed future.
    ///
    /// If `ApiPath.call` returns an Err, it will be converted into a `Response` here.
    pub(crate) fn call(
        &self,
        meta: &ApiRequest,
        req: Request<Body>,
    ) -> Pin<Box<ApiOperationFuture>> {
        match self.paths.get(req.uri().path()) {
            Some(path) => {
                let f = path.call(meta, req);
                let fut = async {
                    match f.await {
                        Ok(res) => Ok(res),
                        Err(e) => Ok(e.into_response()),
                    }
                };
                Box::pin(fut)
            }
            None => {
                let res = ApiError::not_found(1, "apiPathNotFound").into_response();
                let fut = async { Ok(res) };
                Box::pin(fut)
            }
        }
    }
}

/// API request.
#[derive(Clone)]
pub struct ApiRequest {
    /// Thread-safe reference to `Api`.
    api: Arc<Api>,
    /// Date and time of this request.
    date: DateTime<Utc>,
    /// Remote address of this request.
    remote_addr: SocketAddr,
}

impl ApiRequest {
    /// Returns new `ApiRequest`.
    pub fn new(api: Arc<Api>, remote_addr: SocketAddr) -> Self {
        Self {
            api,
            date: Utc::now(),
            remote_addr,
        }
    }

    /// Returns [SocketAddr].
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }
}

impl tower_service::Service<Request<Body>> for ApiRequest {
    type Response = Response<Body>;
    type Error = ApiErrors;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        self.api.call(self, req)
    }
}
