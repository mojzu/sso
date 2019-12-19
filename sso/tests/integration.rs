use sso::{pb::Empty, ClientBlocking, ClientOptions};

#[test]
#[ignore]
fn test_ping() {
    let mut client =
        ClientBlocking::connect(&ClientOptions::new("http://0.0.0.0:7000").authorisation(""))
            .unwrap();
    let request = tonic::Request::new(Empty {});
    let response = client.ping(request).unwrap();
    println!("RESPONSE={:?}", response);
}

// #[macro_use]
// extern crate lazy_static;

// TODO(refactor): Reimplement these tests.
// mod support;

// use support::*;

// TODO(test): Test TLS functionality.
// TODO(test): Password reset tests, use lettre file transport.
// Service 2 cannot confirm reset password.
// Confirm reset password success.
// User password is updated.
// Cannot reuse token.
// Register tests, multiple services, etc.

// audit_integration_test!();
// auth_csrf_integration_test!();
// auth_key_integration_test!();
// auth_local_integration_test!();
// auth_token_integration_test!();
// auth_totp_integration_test!();
// guide_integration_test!();
// key_integration_test!();
// service_integration_test!();
// user_integration_test!();

// #[test]
// #[ignore]
// fn api_ping_ok() {
//     let client = client_create(None);
//     let res = client.ping().unwrap();
//     assert_eq!(res, Value::String("pong".to_owned()));
// }

// #[test]
// #[ignore]
// fn api_metrics_unauthorised() {
//     let client = client_create(Some(INVALID_KEY));
//     let res = client.metrics().unwrap_err();
//     assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED.as_u16());
// }

// #[test]
// #[ignore]
// fn api_metrics_ok() {
//     let client = client_create(None);
//     let res = client.metrics().unwrap();
//     assert!(res.len() > 0);
// }
