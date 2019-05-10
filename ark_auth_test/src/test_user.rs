use crate::support;
use actix_http_test::TestServerRuntime;
use actix_web::http::{header, StatusCode};
use ark_auth::{driver::Driver, server};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
struct UserPassword {
    password: Option<String>,
    password_hash: Option<String>,
    password_revision: Option<i64>,
}

pub fn list_authorisation_test(_driver: &Driver, app: &mut TestServerRuntime) {
    support::get_authorisation_test(app, "/v1/user")
}

pub fn create_authorisation_test(_driver: &Driver, app: &mut TestServerRuntime) {
    let payload = r#"{ "name": "test", "email": "test@example.com" }"#.as_bytes();
    support::post_authorisation_test(app, "/v1/user", payload)
}

pub fn create_test(driver: &Driver, app: &mut TestServerRuntime) {
    let (_service, key) = support::service_key(driver);

    let before = Utc::now();
    let payload = r#"{
        "name": "John Smith",
        "email": "john.smith@example.com",
        "password": "guest"
    }"#
    .as_bytes();
    let req = app
        .post("/v1/user")
        .header(header::CONTENT_TYPE, header::ContentType::json())
        .header(header::AUTHORIZATION, key.value)
        .send_body(payload);

    // 200 OK response.
    let mut res = app.block_on(req).unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = app.block_on(res.body()).unwrap();
    let body: server::route::user::CreateResponse = serde_json::from_slice(&bytes).unwrap();
    assert!(body.data.created_at.gt(&before));
    assert!(body.data.updated_at.gt(&before));
    assert!(body.data.id > 0);
    assert_eq!(body.data.name, "John Smith");
    assert_eq!(body.data.email, "john.smith@example.com");

    // Test password/hash/revision is not leaked.
    let password_leak = serde_json::from_slice::<UserPassword>(&bytes).unwrap();
    assert!(password_leak.password.is_none());
    assert!(password_leak.password_hash.is_none());
    assert!(password_leak.password_revision.is_none());
}

// TODO(test): User tests.
