use crate::support;
use actix_http_test::TestServerRuntime;
use ark_auth::driver::Driver;

pub fn read_authorisation_test(_driver: &Driver, app: &mut TestServerRuntime) {
    support::get_authorisation_test(app, "/v1/service/1")
}

// TODO(test): Service tests.
