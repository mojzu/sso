use crate::support;
use actix_http_test::TestServerRuntime;
use actix_web::http::StatusCode;
use ark_auth::{driver::Driver, server};

pub fn verify_authorisation_test(_driver: &Driver, app: &mut TestServerRuntime) {
    let payload = r#"{ "key": "5a044d9035334e95a60ac0338904d37c" }"#.as_bytes();
    support::post_authorisation_test(app, "/v1/auth/key/verify", payload)
}

pub fn verify_body_test(driver: &Driver, app: &mut TestServerRuntime) {
    let (_service, key) = support::service_key(driver);

    // Invalid body (missing key property).
    // 400 BAD REQUEST response.
    let payload = r#"{}"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/key/verify", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid body (invalid key property).
    // 400 BAD REQUEST response.
    let payload = r#"{ "key": "" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/key/verify", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

pub fn verify_test(driver: &Driver, app: &mut TestServerRuntime) {
    let (service, key) = support::service_key(driver);
    let (_service2, key2) = support::service_key(driver);
    let (user, user_key) = support::user_key(driver, &service, None);

    // Service 2 cannot verify key.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"key": "{}"}}"#, &user_key.value);
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/key/verify", Some(&key2.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Service key cannot be verified.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"key": "{}"}}"#, &key.value);
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/key/verify", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Service verifies key.
    // 200 OK response.
    let payload = format!(r#"{{"key": "{}"}}"#, &user_key.value);
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/key/verify", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::OK);
    assert_eq!(content_length, bytes.len());

    let body: server::route::auth::KeyResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.data.user_id, user.id);
    assert_eq!(body.data.key, user_key.value);
}

pub fn revoke_authorisation_test(_driver: &Driver, app: &mut TestServerRuntime) {
    let payload = r#"{ "key": "5a044d9035334e95a60ac0338904d37c" }"#.as_bytes();
    support::post_authorisation_test(app, "/v1/auth/key/revoke", payload)
}

pub fn revoke_body_test(driver: &Driver, app: &mut TestServerRuntime) {
    let (_service, key) = support::service_key(driver);

    // Invalid body (missing token property).
    // 400 BAD REQUEST response.
    let payload = r#"{}"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/key/revoke", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid body (invalid token property).
    // 400 BAD REQUEST response.
    let payload = r#"{ "token": "" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/key/revoke", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

pub fn revoke_test(driver: &Driver, app: &mut TestServerRuntime) {
    let (service, key) = support::service_key(driver);
    let (_service2, key2) = support::service_key(driver);
    let (_user, user_key) = support::user_key(driver, &service, None);

    // Service 2 cannot revoke key.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"key": "{}"}}"#, &user_key.value);
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/key/revoke", Some(&key2.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Service key cannot be revoked.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"key": "{}"}}"#, &key.value);
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/key/revoke", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Service revokes key.
    // 200 OK response.
    let payload = format!(r#"{{"key": "{}"}}"#, &user_key.value);
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/key/revoke", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::OK);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Key is invalid.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"key": "{}"}}"#, &user_key.value);
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/key/verify", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

// TODO(test): Authentication key tests.
