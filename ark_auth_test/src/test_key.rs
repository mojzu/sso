use crate::support;
use actix_http_test::TestServerRuntime;

pub fn list_authorisation_test(app: &mut TestServerRuntime) {
    support::get_authorisation_test(app, "/v1/key")
}
