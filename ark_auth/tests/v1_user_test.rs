#[macro_use]
extern crate serde_derive;

mod support;

use actix_http::http::header::ContentType;
use actix_web::http::{header, StatusCode};
use ark_auth::api;
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
struct UserPassword {
    password: String,
}

#[test]
fn get_many_user_authorisation_test() {
    let (_db, mut app) = support::app();
    support::get_authorisation_test(&mut app, "/v1/user")
}

#[test]
fn post_user_authorisation_test() {
    let (_db, mut app) = support::app();
    let payload = r#"{ "name": "test", "email": "test@example.com" }"#.as_bytes();
    support::post_authorisation_test(&mut app, "/v1/user", payload)
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
