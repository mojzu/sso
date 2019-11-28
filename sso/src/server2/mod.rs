use crate::DriverResult;
use gotham::{
    router::{builder::*, Router},
    state::State,
};
// use hyper::Method;

pub fn server2_start() -> DriverResult<()> {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router());
    Ok(())
}

fn router() -> Router {
    build_simple_router(|route| {
        // route.request(vec![Get, Head], "/").to(index);

        route.scope("/v1", |route| {
            route.get("/ping").to(v1_ping);
        });
    })
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
