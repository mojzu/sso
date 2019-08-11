#[macro_use]
extern crate lazy_static;

mod support;

use support::*;

// TODO(test): Finish, improve tests.
// TODO(test): Test TLS functionality.
// TODO(test): Password reset tests, SMTP testing using mailin_embedded?
// Service 2 cannot confirm reset password.
// Confirm reset password success.
// User password is updated.
// Cannot reuse token.

audit_integration_test!();
auth_key_integration_test!();
auth_local_integration_test!();
auth_token_integration_test!();
guide_integration_test!();
key_integration_test!();
service_integration_test!();
user_integration_test!();

#[test]
#[ignore]
fn api_ping_ok() {
    let client = client_create(None);
    let res = client.ping().unwrap();
    assert_eq!(res, Value::String("pong".to_owned()));
}

#[test]
#[ignore]
fn api_metrics_forbidden() {
    let client = client_create(Some(INVALID_SERVICE_KEY));
    let res = client.metrics().unwrap_err();
    assert_eq!(res, ClientError::Forbidden);
}

#[test]
#[ignore]
fn api_metrics_ok() {
    let client = client_create(None);
    let res = client.metrics().unwrap();
    assert!(res.len() > 0);
}
