#[macro_use]
extern crate serde_derive;

use futures_util::future::poll_fn;
use http::Method;
use hyper::{Body, Request, Response, StatusCode};
use sso_openapi::*;

pub struct Ping;

impl ApiOperation for Ping {
    fn method(&self) -> Method {
        Method::GET
    }

    fn call(&self, meta: &ApiRequest, req: Request<Body>) -> Box<ApiOperationFuture> {
        let addr = meta.remote_addr();
        let fut = async move {
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

pub struct Openapi;

impl ApiOperation for Openapi {
    fn method(&self) -> Method {
        Method::GET
    }

    fn call(&self, meta: &ApiRequest, req: Request<Body>) -> Box<ApiOperationFuture> {
        // create the body
        let body = Body::from(format!("Hello, {}!", meta.remote_addr()));
        // Create the HTTP response
        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .expect("Unable to create `http::Response`");
        // create a response in a future.
        let fut = async {
            // let b = poll_fn(move |_| {
            //     tokio_executor::threadpool::blocking(|| {
            //         println!("print from blocking");
            //     })
            //     .map_err(|_| panic!("the threadpool shut down"))
            // })
            // .await;

            let _a = Api::blocking::<(), (), _>(|| {
                println!("print from blocking");
                Ok(())
            })
            .await;

            Ok(resp)
        };
        Box::new(fut)
    }
}

impl ApiOperationInto for Openapi {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExQuery {
    x: Option<i64>,
}

pub struct Ex {
    i: i64,
}

impl Ex {
    async fn call_inner(i: &i64, meta: &ApiRequest, req: Request<Body>) -> Result<Response<Body>, ApiErrors> {
        let query = Api::query_parameters::<ExQuery>(&req)?;
        println!("{:?}", query);

        let body = Api::body::<ExQuery>(req.into_body()).await?;
        println!("{:?}", body);

        let r = Api::blocking::<i64, (), _>(|| {
            println!("print from blocking");
            Ok(1)
        })
        .await;
        println!("{:?}", r);

        let body = Body::from(format!("Hello, {}! {}", meta.remote_addr(), i));
        let res = Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .expect("Unable to create `http::Response`");
        Ok(res)
    }
}

impl ApiOperationInto for Ex {}

impl ApiOperation for Ex {
    fn method(&self) -> Method {
        Method::POST
    }

    fn call(&self, meta: &ApiRequest, req: Request<Body>) -> Box<ApiOperationFuture> {
        let i = self.i;
        let meta = meta.clone();
        let f = async move {
            Self::call_inner(&i, &meta, req).await
        };
        Box::new(f)
    }
}

fn main() {
    let addr = "127.0.0.1:1337".parse().unwrap();

    let api = Api::new(vec![
        ApiPath::new("/v1/ping").operation(Ping),
        ApiPath::new("/openapi.json").operation(Openapi),
        ApiPath::new("/ex").operation(Ex { i: 64 }),
    ]);

    println!("Listening on http://{}", addr);
    if let Err(e) = api.run(&addr) {
        eprintln!("Error: {}", e);
    }
}
