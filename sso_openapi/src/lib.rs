#[macro_use]
extern crate serde_derive;

mod error;

pub use error::{ApiError, ApiErrors};

use chrono::{DateTime, Utc};
use futures_util::future::poll_fn;
use http::Method;
use hyper::server::conn::AddrStream;
use hyper::service::make_service_fn;
use hyper::{Body, Request, Response, Server, StatusCode};
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
pub trait ApiOperationInto: 'static + ApiOperation + Sized {
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

    /// Deserialise request URL query parameters into type.
    pub fn query_parameters<'de, T>(req: &'de Request<Body>) -> Result<T, ApiErrors>
    where
        T: serde::de::Deserialize<'de>,
    {
        let i = req.uri().query().unwrap_or("");
        serde_urlencoded::from_str::<T>(i).map_err(|e| ApiErrors::new(e))
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

// Operations.

#[derive(Debug, Serialize, Deserialize)]
pub struct PingQuery {
    x: Option<i64>,
}

pub struct Ping;

impl ApiOperation for Ping {
    fn method(&self) -> Method {
        Method::GET
    }

    fn call(&self, meta: &ApiRequest, req: Request<Body>) -> Box<ApiOperationFuture> {
        let addr = meta.remote_addr;
        let fut = async move {
            let query = Api::query_parameters::<PingQuery>(&req);

            if let Err(e) = query {
                println!("{:?}", e);
                return Err(e);
            }
            println!("{:?}", query);

            // create the body
            let body = Body::from(format!("Hello, {}!", addr));
            // Create the HTTP response
            let resp = Response::builder()
                .status(StatusCode::OK)
                .body(body)
                .expect("Unable to create `http::Response`");
            Ok(resp)
        };
        Box::new(fut)
    }
}

impl ApiOperationInto for Ping {}

pub struct Openapi {
    i: i64,
}

impl ApiOperation for Openapi {
    fn method(&self) -> Method {
        Method::GET
    }

    fn call(&self, meta: &ApiRequest, req: Request<Body>) -> Box<ApiOperationFuture> {
        // create the body
        let body = Body::from(format!("Hello, {}! {}", meta.remote_addr, self.i));
        // Create the HTTP response
        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .expect("Unable to create `http::Response`");
        // create a response in a future.
        let fut = async {
            let b = poll_fn(move |_| {
                tokio_executor::threadpool::blocking(|| {
                    println!("print from blocking");
                })
                .map_err(|_| panic!("the threadpool shut down"))
            })
            .await;
            Ok(resp)
        };
        Box::new(fut)
    }
}

impl ApiOperationInto for Openapi {}

pub fn run() {
    let addr = "127.0.0.1:1337".parse().unwrap();

    let svc = Api::new(vec![
        ApiPath::new("/v1/ping").operation(Ping),
        ApiPath::new("/openapi.json").operation(Openapi { i: 64 }),
    ]);

    let svc = Arc::new(svc);

    let make_svc = make_service_fn(move |socket: &AddrStream| {
        let request = ApiRequest::new(svc.clone(), socket.remote_addr());
        async move { Ok::<_, http::Error>(request) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    let rt = tokio::runtime::Runtime::new().unwrap();
    if let Err(e) = rt.block_on(server) {
        eprintln!("server error: {}", e);
    }
}
