mod support;

use actix_web::http::StatusCode;
use ark_auth::api;

#[test]
fn post_key_verify_authorisation_test() {
    let (_db, mut app) = support::app();

    // Missing header.
    // 403 FORBIDDEN response.
    let payload = r#"{ "key": "5a044d9035334e95a60ac0338904d37c" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(&mut app, "/v1/auth/key/verify", None, payload);
    assert_eq!(status_code, StatusCode::FORBIDDEN);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid header.
    // 403 FORBIDDEN response.
    let (status_code, content_length, bytes) =
        support::app_post(&mut app, "/v1/auth/key/verify", Some("invalid"), payload);
    assert_eq!(status_code, StatusCode::FORBIDDEN);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

#[test]
fn post_key_verify_body_test() {
    let (db, mut app) = support::app();
    let (_service, key) = support::service_key(&db);

    // Invalid body (missing key property).
    // 400 BAD REQUEST response.
    let payload = r#"{}"#.as_bytes();
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/key/verify",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid body (invalid key property).
    // 400 BAD REQUEST response.
    let payload = r#"{ "key": "" }"#.as_bytes();
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/key/verify",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

#[test]
fn post_key_verify_test() {
    let (db, mut app) = support::app();
    let (service, key) = support::service_key(&db);
    let (_service2, key2) = support::service_key(&db);
    let (user, user_key) = support::user_key(&db, &service, None);

    // Service 2 cannot verify key.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"key": "{}"}}"#, &user_key.key_value);
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/key/verify",
        Some(&key2.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Service key cannot be verified.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"key": "{}"}}"#, &key.key_value);
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/key/verify",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Service verifies key.
    // 200 OK response.
    let payload = format!(r#"{{"key": "{}"}}"#, &user_key.key_value);
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/key/verify",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::OK);
    assert_eq!(content_length, bytes.len());

    let body: api::auth::KeyResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.user_id, user.user_id);
    assert_eq!(body.key, user_key.key_value);
}

#[test]
fn post_key_revoke_authorisation_test() {
    let (_db, mut app) = support::app();

    // Missing header.
    // 403 FORBIDDEN response.
    let payload = r#"{ "key": "5a044d9035334e95a60ac0338904d37c" }"#.as_bytes();
    let (status_code, content_length, bytes) =
        support::app_post(&mut app, "/v1/auth/key/revoke", None, payload);
    assert_eq!(status_code, StatusCode::FORBIDDEN);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid header.
    // 403 FORBIDDEN response.
    let (status_code, content_length, bytes) =
        support::app_post(&mut app, "/v1/auth/key/revoke", Some("invalid"), payload);
    assert_eq!(status_code, StatusCode::FORBIDDEN);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

#[test]
fn post_key_revoke_body_test() {
    let (db, mut app) = support::app();
    let (_service, key) = support::service_key(&db);

    // Invalid body (missing token property).
    // 400 BAD REQUEST response.
    let payload = r#"{}"#.as_bytes();
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/key/revoke",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid body (invalid token property).
    // 400 BAD REQUEST response.
    let payload = r#"{ "token": "" }"#.as_bytes();
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/key/revoke",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

#[test]
fn post_key_revoke_test() {
    let (db, mut app) = support::app();
    let (service, key) = support::service_key(&db);
    let (_service2, key2) = support::service_key(&db);
    let (_user, user_key) = support::user_key(&db, &service, None);

    // Service 2 cannot revoke key.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"key": "{}"}}"#, &user_key.key_value);
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/key/revoke",
        Some(&key2.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Service key cannot be revoked.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"key": "{}"}}"#, &key.key_value);
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/key/revoke",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Service revokes key.
    // 200 OK response.
    let payload = format!(r#"{{"key": "{}"}}"#, &user_key.key_value);
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/key/revoke",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::OK);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Key is invalid.
    // 400 BAD REQUEST response.
    let payload = format!(r#"{{"key": "{}"}}"#, &user_key.key_value);
    let (status_code, content_length, bytes) = support::app_post(
        &mut app,
        "/v1/auth/key/verify",
        Some(&key.key_value),
        payload,
    );
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}
