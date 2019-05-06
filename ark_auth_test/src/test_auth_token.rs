use crate::support;
use actix_http_test::TestServerRuntime;
use actix_web::http::StatusCode;
use ark_auth::{core, driver::Driver, server};

pub fn verify_authorisation_test(_driver: &Driver, app: &mut TestServerRuntime) {
    let payload = r#"{ "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9" }"#.as_bytes();
    support::post_authorisation_test(app, "/v1/auth/token/verify", payload)
}

pub fn verify_body_test(driver: &Driver, app: &mut TestServerRuntime) {
    let (_service, key) = support::service_key(driver);

    // Invalid body (missing token property).
    // 400 BAD REQUEST response.
    let payload = r#"{}"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/token/verify", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid body (invalid token property).
    // 400 BAD REQUEST response.
    let payload = r#"{ "token": "" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/token/verify", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

pub fn verify_test(driver: &Driver, app: &mut TestServerRuntime) {
    let (service, key) = support::service_key(driver);
    let (_service2, key2) = support::service_key(driver);
    let (user, _user_key) = support::user_key(driver, &service, Some("guest"));
    let token = core::auth::login(driver, &service, &user.email, "guest").unwrap();

    // Service 2 cannot verify token.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"token": "{}"}}"#, &token.token);
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/token/verify", Some(&key2.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Service verifies token.
    // 200 OK response.
    let payload = format!(r#"{{"token": "{}"}}"#, &token.token);
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/token/verify", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::OK);
    assert_eq!(content_length, bytes.len());

    let body: server::auth::TokenResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.data.user_id, user.id);
    assert_eq!(body.data.token, token.token);
    assert_eq!(body.data.token_expires, token.token_expires);
}

pub fn refresh_authorisation_test(_driver: &Driver, app: &mut TestServerRuntime) {
    let payload = r#"{ "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9" }"#.as_bytes();
    support::post_authorisation_test(app, "/v1/auth/token/refresh", payload)
}

pub fn refresh_body_test(driver: &Driver, app: &mut TestServerRuntime) {
    let (_service, key) = support::service_key(driver);

    // Invalid body (missing token property).
    // 400 BAD REQUEST response.
    let payload = r#"{}"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/token/refresh", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid body (invalid token property).
    // 400 BAD REQUEST response.
    let payload = r#"{ "token": "" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/token/refresh", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

pub fn refresh_test(driver: &Driver, app: &mut TestServerRuntime) {
    let (service, key) = support::service_key(driver);
    let (_service2, key2) = support::service_key(driver);
    let (user, _user_key) = support::user_key(driver, &service, Some("guest"));
    let token = core::auth::login(driver, &service, &user.email, "guest").unwrap();

    // Sleep to ensure refreshed tokens have different expiry time.
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Service 2 cannot refresh token.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"token": "{}"}}"#, &token.token);
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/token/refresh", Some(&key2.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Service refreshes token.
    // 200 OK response.
    let payload = format!(r#"{{"token": "{}"}}"#, &token.token);
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/token/refresh", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::OK);
    assert_eq!(content_length, bytes.len());

    let body: server::auth::TokenResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.data.user_id, user.id);
    assert_ne!(body.data.token, token.token);
    assert!(body.data.token_expires > token.token_expires);
}

pub fn revoke_authorisation_test(_driver: &Driver, app: &mut TestServerRuntime) {
    let payload = r#"{ "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9" }"#.as_bytes();
    support::post_authorisation_test(app, "/v1/auth/token/revoke", payload)
}

pub fn revoke_body_test(driver: &Driver, app: &mut TestServerRuntime) {
    let (_service, key) = support::service_key(driver);

    // Invalid body (missing token property).
    // 400 BAD REQUEST response.
    let payload = r#"{}"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/token/revoke", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid body (invalid token property).
    // 400 BAD REQUEST response.
    let payload = r#"{ "token": "" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/token/revoke", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

pub fn revoke_test(driver: &Driver, app: &mut TestServerRuntime) {
    let (service, key) = support::service_key(driver);
    let (_service2, key2) = support::service_key(driver);
    let (user, _user_key) = support::user_key(driver, &service, Some("guest"));
    let token = core::auth::login(driver, &service, &user.email, "guest").unwrap();

    // Service 2 cannot revoke token.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"token": "{}"}}"#, &token.token);
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/token/revoke", Some(&key2.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Service revokes token.
    // 200 OK response.
    let payload = format!(r#"{{"token": "{}"}}"#, &token.token);
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/token/revoke", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::OK);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Token is invalid.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"token": "{}"}}"#, &token.token);
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/token/verify", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}
