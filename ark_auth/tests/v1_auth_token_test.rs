mod support;

use actix_web::http::{header, StatusCode};
use ark_auth::api;

#[test]
fn post_token_verify_authorisation_test() {
    let (_db, mut app) = support::app();
    let payload = r#"{ "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9" }"#.as_bytes();
    support::post_authorisation_test(&mut app, "/v1/auth/token/verify", payload)
}

#[test]
fn post_token_verify_body_test() {
    let (db, mut app) = support::app();
    let (_service, key) = support::service_key(&db);

    // Invalid body (missing token property).
    let payload = r#"{}"#.as_bytes();
    let req = app
        .post("/v1/auth/token/verify")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    // Invalid body (invalid token property).
    let payload = r#"{ "token": "" }"#.as_bytes();
    let req = app
        .post("/v1/auth/token/verify")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");
}

#[test]
fn post_token_verify_test() {
    let (db, mut app) = support::app();
    let (service, key) = support::service_key(&db);
    let (_service2, key2) = support::service_key(&db);
    let (user, user_key) = support::user_key(&db, &service, None);
    let token = ark_auth::db::auth::login(&user, &user_key, &service).unwrap();

    // Service 2 cannot verify token.
    let payload = format!(r#"{{"token": "{}"}}"#, &token.token);
    let req = app
        .post("/v1/auth/token/verify")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key2.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    // Service verifies token.
    let payload = format!(r#"{{"token": "{}"}}"#, &token.token);
    let req = app
        .post("/v1/auth/token/verify")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 200 OK response.
    let mut res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = app.block_on(res.body()).unwrap();
    let body: api::auth::TokenResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.data.user_id, user.user_id);
    assert_eq!(body.data.token, token.token);
    assert_eq!(body.data.token_expires, token.token_expires);
}

#[test]
fn post_token_refresh_authorisation_test() {
    let (_db, mut app) = support::app();
    let payload = r#"{ "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9" }"#.as_bytes();
    support::post_authorisation_test(&mut app, "/v1/auth/token/refresh", payload)
}

#[test]
fn post_token_refresh_body_test() {
    let (db, mut app) = support::app();
    let (_service, key) = support::service_key(&db);

    // Invalid body (missing token property).
    let payload = r#"{}"#.as_bytes();
    let req = app
        .post("/v1/auth/token/refresh")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    // Invalid body (invalid token property).
    let payload = r#"{ "token": "" }"#.as_bytes();
    let req = app
        .post("/v1/auth/token/refresh")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");
}

#[test]
fn post_token_refresh_test() {
    let (db, mut app) = support::app();
    let (service, key) = support::service_key(&db);
    let (_service2, key2) = support::service_key(&db);
    let (user, user_key) = support::user_key(&db, &service, None);
    let token = ark_auth::db::auth::login(&user, &user_key, &service).unwrap();

    // Sleep to ensure refreshed tokens have different expiry time.
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Service 2 cannot refresh token.
    let payload = format!(r#"{{"token": "{}"}}"#, &token.token);
    let req = app
        .post("/v1/auth/token/refresh")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key2.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    // Service refreshes token.
    let payload = format!(r#"{{"token": "{}"}}"#, &token.token);
    let req = app
        .post("/v1/auth/token/refresh")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 200 OK response.
    let mut res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = app.block_on(res.body()).unwrap();
    let body: api::auth::TokenResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.data.user_id, user.user_id);
    assert_ne!(body.data.token, token.token);
    assert!(body.data.token_expires > token.token_expires);
}

#[test]
fn post_token_revoke_authorisation_test() {
    let (_db, mut app) = support::app();
    let payload = r#"{ "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9" }"#.as_bytes();
    support::post_authorisation_test(&mut app, "/v1/auth/token/revoke", payload)
}

#[test]
fn post_token_revoke_body_test() {
    let (db, mut app) = support::app();
    let (_service, key) = support::service_key(&db);

    // Invalid body (missing token property).
    let payload = r#"{}"#.as_bytes();
    let req = app
        .post("/v1/auth/token/revoke")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    // Invalid body (invalid token property).
    let payload = r#"{ "token": "" }"#.as_bytes();
    let req = app
        .post("/v1/auth/token/revoke")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");
}

#[test]
fn post_token_revoke_test() {
    let (db, mut app) = support::app();
    let (service, key) = support::service_key(&db);
    let (_service2, key2) = support::service_key(&db);
    let (user, user_key) = support::user_key(&db, &service, None);
    let token = ark_auth::db::auth::login(&user, &user_key, &service).unwrap();

    // Service 2 cannot revoke token.
    let payload = format!(r#"{{"token": "{}"}}"#, &token.token);
    let req = app
        .post("/v1/auth/token/revoke")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key2.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    // Service revokes token.
    let payload = format!(r#"{{"token": "{}"}}"#, &token.token);
    let req = app
        .post("/v1/auth/token/revoke")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 200 OK response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    // Token is invalid.
    let payload = format!(r#"{{"token": "{}"}}"#, &token.token);
    let req = app
        .post("/v1/auth/token/verify")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.key_value.to_owned())
        .send_body(payload);

    // 400 BAD REQUEST response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");
}
