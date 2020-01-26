#[macro_use]
extern crate serde_json;

mod support;

use support::*;

// TODO(test): Test TLS functionality.
// TODO(test): Password reset tests, use lettre file transport.
// TODO(test): Service 2 cannot confirm reset password.
// TODO(test): Confirm reset password success.
// TODO(test): User password is updated.
// TODO(test): Cannot reuse token.
// TODO(test): Refresh token exchange between services.

audit_integration_test!();
auth_csrf_integration_test!();
auth_key_integration_test!();
auth_local_integration_test!();
auth_token_integration_test!();
auth_totp_integration_test!();
guide_integration_test!();
key_integration_test!();
service_integration_test!();
user_integration_test!();

#[test]
#[ignore]
fn ping_not_found() {
    let mut client = client_create(None);
    let res = client.ping(()).unwrap_err();
    assert_eq!(res.code(), tonic::Code::NotFound);
}

#[test]
#[ignore]
fn metrics_not_found() {
    let mut client = client_create(Some(INVALID_KEY));
    let res = client.metrics(()).unwrap_err();
    assert_eq!(res.code(), tonic::Code::NotFound);
}
