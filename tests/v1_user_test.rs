#[macro_use]
extern crate serde_derive;

mod support;

use actix_http::http::header::ContentType;
use actix_web::http::{header, StatusCode};
use chrono::Utc;
use mr_auth::api;

#[derive(Debug, Serialize, Deserialize)]
struct UserPassword {
    password: String,
}

#[test]
fn post_user_authorisation_test() {
    let (_db, mut app) = support::app();

    // Missing header.
    let payload = r#"{ "name": "User Name", "email": "user1@example.com" }"#.as_bytes();
    let req = app
        .post("/v1/user")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .send_body(payload);

    // 403 FORBIDDEN response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");

    // Invalid header.
    let req = app
        .post("/v1/user")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, "invalid")
        .send_body(payload);

    // 403 FORBIDDEN response.
    let res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    assert_eq!(res.headers().get(header::CONTENT_LENGTH).unwrap(), "0");
}

#[test]
fn post_user_test() {
    let (db, mut app) = support::app();
    let (_service, key) = support::service_key(&db);

    let before = Utc::now();
    let payload = r#"{
        "name": "John Smith",
        "email": "john.smith@example.com",
        "password": "guest"
    }"#
    .as_bytes();
    let req = app
        .post("/v1/user")
        .header(header::CONTENT_TYPE, ContentType::json())
        .header(header::AUTHORIZATION, key.key_value)
        .send_body(payload);

    // 200 OK response.
    let mut res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = app.block_on(res.body()).unwrap();
    let body: api::user::User = serde_json::from_slice(&bytes).unwrap();
    assert!(body.created_at.gt(&before));
    assert!(body.updated_at.gt(&before));
    assert!(body.id > 0);
    assert_eq!(body.name, "John Smith");
    assert_eq!(body.email, "john.smith@example.com");

    // Test password is not leaked.
    let password_leak = serde_json::from_slice::<UserPassword>(&bytes);
    assert!(password_leak.is_err());
}
