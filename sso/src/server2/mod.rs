use crate::DriverResult;
use gotham::{
    middleware::state::StateMiddleware,
    pipeline::{single::single_pipeline, single_middleware},
    router::{builder::*, Router},
    state::{FromState, State},
};
use openapi::{
    v3_0::{Info, Spec},
    OpenApi,
};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
// use hyper::Method;

#[derive(Clone, StateData)]
struct OpenApiState {
    spec: Arc<RwLock<Spec>>,
}

impl OpenApiState {
    /// New OpenAPI state from specification.
    /// Specification is wrapped in [Arc][std::sync::Arc] for sharing between threads.
    /// Specification is wrapped in [RwLock][std::sync::RwLock] to allow mutation by one thread.
    pub fn new(spec: Spec) -> Self {
        Self { spec: Arc::new(RwLock::new(spec)) }
    }

    /// Serialise OpenAPI specification as JSON string.
    pub fn to_json(&self) -> openapi::Result<String> {
        let spec = self.spec.as_ref().read().unwrap();
        openapi::to_json(&OpenApi::V3_0(spec.clone()))
    }
}

pub fn server2_start() -> DriverResult<()> {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router());
    Ok(())
}

fn router() -> Router {
    let spec = Spec {
        openapi: String::from("3.0.2"),
        info: Info {
            title: String::from("sso"),
            description: None,
            terms_of_service: None,
            version: String::from("v1"),
            contact: None,
            license: None,
        },
        servers: None,
        paths: BTreeMap::default(),
        components: None,
        tags: None,
        external_docs: None,
    };

    // build_simple_router(|route| {
    //     // route.request(vec![Get, Head], "/").to(index);
    //     route.scope("/v1", |route| {
    //         route.get("/ping").to(v1_ping);
    //     });
    // })

    let openapi_state = OpenApiState::new(spec);

    // create our state middleware to share the counter
    let middleware = StateMiddleware::new(openapi_state);

    // create a middleware pipeline from our middleware
    let pipeline = single_middleware(middleware);

    // construct a basic chain from our pipeline
    let (chain, pipelines) = single_pipeline(pipeline);

    // build a router with the chain & pipeline
    build_router(chain, pipelines, |route| {
        route.get("/").to(v1_openapi);
    })
}

fn v1_openapi(state: State) -> (State, (mime::Mime, String)) {
    let res = {
        let s = OpenApiState::borrow_from(&state);
        let v = s.to_json().unwrap();
        (mime::APPLICATION_JSON, v)
    };
    (state, res)
}

fn v1_ping(state: State) -> (State, (mime::Mime, Vec<u8>)) {
    let res = {
        (
            mime::APPLICATION_JSON,
            serde_json::to_vec("pong").expect("serialized product"),
        )
    };
    (state, res)
}
