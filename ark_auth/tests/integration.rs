mod support;

use support::*;

// TODO(test): Finish, improve tests.
// TODO(test): Password reset tests, SMTP testing using mailin_embedded?
// Service 2 cannot confirm reset password.
// Confirm reset password success.
// User password is updated.
// Cannot reuse token.

guide_integration_test!();
auth_local_integration_test!();
auth_key_integration_test!();
auth_token_integration_test!();
audit_integration_test!();
key_integration_test!();
service_integration_test!();
user_integration_test!();

#[test]
#[ignore]
fn api_ping_ok() {
    let client = client_create();
    let res = client.ping().unwrap();
    assert_eq!(res, Value::String("pong".to_owned()));
}
