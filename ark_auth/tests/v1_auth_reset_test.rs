mod support;

use actix_web::http::StatusCode;
use ark_auth::api::auth::reset::ResetPasswordConfirmResponse;

#[test]
fn post_reset_password_authorisation_test() {
    let (_db, mut app) = support::app();
    let payload = r#"{ "email": "reset-password@example.com" }"#.as_bytes();
    support::post_authorisation_test(&mut app, "/v1/auth/reset/password", payload)
}

#[test]
fn post_reset_password_body_test() {
    let (db, mut app) = support::app();
    let (_service, key) = support::service_key(&db);

    // Invalid body (missing email property).
    // 400 BAD REQUEST response.
    let payload = r#"{}"#.as_bytes();
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/reset/password",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid body (invalid email property).
    // 400 BAD REQUEST response.
    let payload = r#"{ "email": "reset-password" }"#.as_bytes();
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/reset/password",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

#[test]
fn post_reset_password_test() {
    let (db, mut app) = support::app();
    let (service, key) = support::service_key(&db);
    let (_service2, key2) = support::service_key(&db);

    // Unknown email address.
    // 400 BAD REQUEST response.
    let payload = r#"{ "email": "reset-password@example.com" }"#.as_bytes();
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/reset/password",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    let (user, _key) = support::user_key(&db, &service, Some("guest"));

    // Service 2 cannot reset password.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"email": "{}"}}"#, &user.user_email);
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/reset/password",
        Some(&key2.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Reset password success.
    // 200 OK response.
    let payload = format!(r#"{{"email": "{}"}}"#, &user.user_email);
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/reset/password",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::OK);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

#[test]
fn post_reset_password_confirm_authorisation_test() {
    let (_db, mut app) = support::app();
    let payload = r#"{ "token": "some-token", "password": "guest" }"#.as_bytes();
    support::post_authorisation_test(&mut app, "/v1/auth/reset/password/confirm", payload)
}

#[test]
fn post_reset_password_confirm_body_test() {
    let (db, mut app) = support::app();
    let (_service, key) = support::service_key(&db);

    // Invalid body (missing properties).
    // 400 BAD REQUEST response.
    let payload = r#"{}"#.as_bytes();
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/reset/password/confirm",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid body (invalid token property).
    // 400 BAD REQUEST response.
    let payload = r#"{ "token": "", "password": "guest" }"#.as_bytes();
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/reset/password/confirm",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid body (invalid password property).
    // 400 BAD REQUEST response.
    let payload = r#"{ "token": "some-token", "password": "" }"#.as_bytes();
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/reset/password/confirm",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

#[test]
fn post_reset_password_confirm_test() {
    let (db, mut app) = support::app();
    let (service, key) = support::service_key(&db);
    let (_servic2, key2) = support::service_key(&db);
    let (user, _key) = support::user_key(&db, &service, Some("guest"));
    let (_, token) = db.auth_reset_password(&user.user_email, &service).unwrap();

    // Service 2 cannot confirm reset password.
    // 400 BAD REQUEST response.
    let payload = format!(
        r#"{{"token": "{}", "password": "guestguest"}}"#,
        &token.token
    );
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/reset/password/confirm",
        Some(&key2.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Confirm reset password success.
    // 200 OK response.
    let payload = format!(
        r#"{{"token": "{}", "password": "guestguest"}}"#,
        &token.token
    );
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/reset/password/confirm",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::OK);
    assert_eq!(content_length, bytes.len());

    let body: ResetPasswordConfirmResponse = serde_json::from_slice(&bytes).unwrap();
    assert!(body.meta.password_strength.is_some());
    assert_eq!(body.meta.password_pwned, None);

    // User password is updated.
    // 200 OK response.
    let payload = format!(
        r#"{{"email": "{}", "password": "guestguest"}}"#,
        &user.user_email
    );
    let (status_code, _content_length, _bytes) =
        support::app_post(&mut app, "/v1/auth/login", Some(&key.key_value), payload);
    assert_eq!(status_code, StatusCode::OK);

    // Cannot reuse token.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"token": "{}", "password": "guest"}}"#, &token.token);
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/reset/password/confirm",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}
