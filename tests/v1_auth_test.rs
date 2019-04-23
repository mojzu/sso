mod support;

use actix_web::http::{header, StatusCode};
use ark_auth::api;

#[test]
fn post_login_authorisation_test() {
    let (_db, mut app) = support::app();

    // Missing header.
    let payload = r#"{ "email": "login@example.com", "password": "guest" }"#.as_bytes();
    let req = app
        .post("/v1/auth/login")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .send_body(payload);

    // 403 FORBIDDEN response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    // Invalid header.
    let req = app
        .post("/v1/auth/login")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, "invalid")
        .send_body(payload);

    // 403 FORBIDDEN response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");
}

#[test]
fn post_login_body_test() {
    let (db, mut app) = support::app();
    let (_service, key) = support::service_key(&db);

    // Invalid body (missing properties).
    let payload = r#"{}"#.as_bytes();
    let req = app
        .post("/v1/auth/login")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    // Invalid body (missing email property).
    let payload = r#"{ "password": "guest" }"#.as_bytes();
    let req = app
        .post("/v1/auth/login")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    // Invalid body (missing password property).
    let payload = r#"{ "email": "login@example.com" }"#.as_bytes();
    let req = app
        .post("/v1/auth/login")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    // Invalid body (invalid email property).
    let payload = r#"{ "email": "login", "password": "guest" }"#.as_bytes();
    let req = app
        .post("/v1/auth/login")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    // Invalid body (invalid password property).
    let payload = r#"{ "email": "login@example.com", "password": "" }"#.as_bytes();
    let req = app
        .post("/v1/auth/login")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");
}

#[test]
fn post_login_test() {
    let (db, mut app) = support::app();
    let (service, key) = support::service_key(&db);
    let (_service2, key2) = support::service_key(&db);

    // User not created, unknown email address.
    let payload = r#"{ "email": "login@example.com", "password": "guest" }"#.as_bytes();
    let req = app
        .post("/v1/auth/login")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    let user = db
        .user_create("Login", "login@example.com", Some("guest"))
        .unwrap();

    // User created, does not have key for service.
    let payload = r#"{ "email": "login@example.com", "password": "guest" }"#.as_bytes();
    let req = app
        .post("/v1/auth/login")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    db.key_create("login", service.service_id, Some(user.user_id))
        .unwrap();

    // Key created, incorrect password.
    let payload = r#"{ "email": "login@example.com", "password": "guests" }"#.as_bytes();
    let req = app
        .post("/v1/auth/login")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    // Correct password.
    let payload = r#"{ "email": "login@example.com", "password": "guest" }"#.as_bytes();
    let req = app
        .post("/v1/auth/login")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 200 OK response.
    let mut res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = app.block_on(res.body()).unwrap();
    let body: api::auth::TokenResponse = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(body.user_id, user.user_id);
    assert!(body.token.len() > 0);
    assert!(body.token_expires > 0);

    // Service 2 does not have key for user.
    let payload = r#"{ "email": "login@example.com", "password": "guest" }"#.as_bytes();
    let req = app
        .post("/v1/auth/login")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key2.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");
}
