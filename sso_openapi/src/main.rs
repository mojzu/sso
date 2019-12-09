#[macro_use]
extern crate serde_derive;

use futures_util::future::poll_fn;
use http::Method;
use hyper::{Body, Request, Response, StatusCode};
use sso_openapi::*;

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
        let addr = meta.remote_addr();
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
        let body = Body::from(format!("Hello, {}! {}", meta.remote_addr(), self.i));
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

fn main() {
    let addr = "127.0.0.1:1337".parse().unwrap();

    let api = Api::new(vec![
        ApiPath::new("/v1/ping").operation(Ping),
        ApiPath::new("/openapi.json").operation(Openapi { i: 64 }),
    ]);

    println!("Listening on http://{}", addr);
    if let Err(e) = api.run(&addr) {
        eprintln!("Error: {}", e);
    }
}
