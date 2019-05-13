use crate::support;
use actix_http_test::TestServerRuntime;
use actix_web::http::StatusCode;
use ark_auth::{core, driver::Driver, server};

pub fn login_authorisation_test(_driver: &Driver, app: &mut TestServerRuntime) {
    let payload = r#"{ "email": "login@example.com", "password": "guest" }"#.as_bytes();
    support::post_authorisation_test(app, "/v1/auth/login", payload)
}

pub fn login_body_test(driver: &Driver, app: &mut TestServerRuntime) {
    let (_service, key) = support::service_key(driver);

    // Invalid body (missing properties).
    // 400 BAD REQUEST response.
    let payload = r#"{}"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/login", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid body (missing email property).
    // 400 BAD REQUEST response.
    let payload = r#"{ "password": "guest" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/login", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid body (missing password property).
    // 400 BAD REQUEST response.
    let payload = r#"{ "email": "login@example.com" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/login", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid body (invalid email property).
    // 400 BAD REQUEST response.
    let payload = r#"{ "email": "login", "password": "guest" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/login", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid body (invalid password property).
    // 400 BAD REQUEST response.
    let payload = r#"{ "email": "login@example.com", "password": "" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/login", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

pub fn login_test(driver: &Driver, app: &mut TestServerRuntime) {
    let (service, key) = support::service_key(driver);
    let (_service2, key2) = support::service_key(driver);

    // User not created, unknown email address.
    // 400 BAD REQUEST response.
    let payload = r#"{ "email": "login@example.com", "password": "guest" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/login", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    let user = core::user::create(
        driver,
        Some(&service),
        "Login",
        "login@example.com",
        Some("guest"),
    )
    .unwrap();

    // User created, does not have key for service.
    // 400 BAD REQUEST response.
    let payload = r#"{ "email": "login@example.com", "password": "guest" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/login", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    core::key::create_user(driver, "login", service.id, user.id).unwrap();

    // Key created, incorrect password.
    // 400 BAD REQUEST response.
    let payload = r#"{ "email": "login@example.com", "password": "guests" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/login", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Correct password.
    // 200 OK response.
    let payload = r#"{ "email": "login@example.com", "password": "guest" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/login", Some(&key.value), payload);
    assert_eq!(status_code, StatusCode::OK);
    assert_eq!(content_length, bytes.len());

    let body: server::route::auth::TokenResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.data.user_id, user.id);
    assert!(body.data.token.len() > 0);
    assert!(body.data.token_expires > 0);

    // Service 2 does not have key for user.
    // 400 BAD REQUEST response.
    let payload = r#"{ "email": "login@example.com", "password": "guest" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(app, "/v1/auth/login", Some(&key2.value), payload);
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

// TODO(test): Authentication tests.
