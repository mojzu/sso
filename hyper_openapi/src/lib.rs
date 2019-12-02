use chrono::{DateTime, Utc};
use core::pin::Pin;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Request, Response, Server, StatusCode};
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

pub trait SvcFut: Future<Output = Result<Response<Body>, Error>> {}

pub type SvcFutBox = Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send + Sync>>;

pub type SvcFn = Box<dyn Fn(&Svc, Request<Body>, RequestMeta) -> SvcFutBox + Send + Sync>;

// pub type SvcErrFn = Box<dyn FnMut(&Svc, Request<Body>, RequestMeta) -> SvcFutBox + Send + Sync>;

pub type SvcFnFut = Box<dyn Future<Output = Result<Response<Body>, Error>>>;

pub type SvcOpFn = Box<dyn Fn() -> SvcFnFut>;

#[derive(Clone)]
pub struct SvcOp {
    method: String,
    f: Arc<SvcOpFn>,
}

impl SvcOp {
    pub fn new<M>(method: M, f: SvcOpFn) -> Self
    where
        M: Into<String>,
    {
        Self {
            method: method.into(),
            f: Arc::new(f),
        }
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn f(&self) -> SvcFnFut {
        let f = self.f.as_ref();
        f()
    }
}

#[derive(Clone)]
pub struct SvcPath {
    path: String,
    ops: HashMap<String, SvcOp>,
}

impl SvcPath {
    pub fn new<P>(path: P) -> Self
    where
        P: Into<String>,
    {
        Self {
            path: path.into(),
            ops: HashMap::new(),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn op(mut self, op: SvcOp) -> Self {
        self.ops.insert(op.method().to_owned(), op);
        self
    }
}

#[derive(Clone)]
pub struct Svc {
    paths: HashMap<String, SvcPath>,
}

impl Svc {
    pub fn new() -> Self {
        Self {
            paths: HashMap::new(),
        }
    }

    pub fn path(mut self, path: SvcPath) -> Self {
        self.paths.insert(path.path().to_owned(), path);
        self
    }

    // pub fn path_match(&self, req: Request<Body>, meta: RequestMeta) -> SvcFutBox {
    //     match self.paths.get(req.uri().path()) {
    //         Some(path) => {
    //             let f = path.as_ref();
    //             f(self, req, meta)
    //         }
    //         None => {
    //             let f = async {
    //                 Ok(Response::builder()
    //                     .status(StatusCode::NOT_FOUND)
    //                     .body(Body::empty())
    //                     .unwrap())
    //             };
    //             Box::pin(f)
    //         }
    //     }
    // }
}

#[derive(Debug, Copy, Clone)]
pub struct RequestMeta {
    date: DateTime<Utc>,
    remote_addr: SocketAddr,
}

impl RequestMeta {
    pub fn new(remote_addr: SocketAddr) -> Self {
        Self {
            date: Utc::now(),
            remote_addr,
        }
    }
}

fn v1_ping(svc: &Svc, req: Request<Body>, meta: RequestMeta) -> SvcFutBox {
    let a = async move {
        Ok(Response::new(Body::from(format!(
            "Hello, {}!",
            meta.remote_addr
        ))))
    };
    Box::pin(a)
}

// impl Future<Output = Result<Response<Body>, Error>>
fn v1_ping2() -> SvcFnFut {
    let a = async move { Ok(Response::new(Body::from(format!("Hello!",)))) };
    Box::new(a)
}

pub fn run() {
    let addr = "127.0.0.1:1337".parse().unwrap();

    // let svc = Svc::new().path("/v1/ping", Box::new(v1_ping));

    let svc = Svc::new().path(SvcPath::new("/v1/ping").op(SvcOp::new("GET", Box::new(v1_ping2))));

    let svc = Arc::new(svc);

    let make_svc = make_service_fn(move |socket: &AddrStream| {
        let svc = svc.clone();
        let remote_addr = socket.remote_addr();
        async move {
            Ok::<_, Error>(service_fn(move |req: Request<Body>| {
                // let svc = svc.clone();
                // // let f = svc_request(svc.as_ref(), req, RequestMeta::new(remote_addr));
                // // Box::pin(f)
                // let f = async move {
                //     let s = svc
                //         .as_ref()
                //         .path_match(req, RequestMeta::new(remote_addr))
                //         .await;
                //     s
                // };
                // Box::pin(f)
                async move {
                    Ok::<_, Error>(Response::new(Body::from(format!(
                        "Hello, {}!",
                        remote_addr
                    ))))
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    let rt = tokio::runtime::Runtime::new().unwrap();
    if let Err(e) = rt.block_on(server) {
        eprintln!("server error: {}", e);
    }
}

// let service = make_service_fn(|_| {
//     async {
//         Ok::<_, hyper::Error>(service_fn(echo))
//     }
// });

// use hyper::{Body, Error, Request, Response, Server};
// use hyper::rt::{self, Future};
//
// let make_svc = make_service_fn(|socket: &AddrStream| {
//     let remote_addr = socket.remote_addr();
//     async move {
//         Ok::<_, Error>(service_fn(move |_: Request<Body>| async move {
//             Ok::<_, Error>(
//                 Response::new(Body::from(format!("Hello, {}!", remote_addr)))
//             )
//         }))
//     }
// });

// let new_service = make_service_fn(move |_| {
//     // Move a clone of `client` into the `service_fn`.
//     let client = client.clone();
//     async {
//         Ok::<_, GenericError>(service_fn(move |req| {
//             // Clone again to ensure that client outlives this closure.
//             response_examples(req, client.to_owned())
//         }))
//     }
// });
