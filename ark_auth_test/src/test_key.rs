use crate::support;
use actix_http_test::TestServerRuntime;
use actix_web::http::StatusCode;
use ark_auth::{driver::Driver, server};

pub fn list_test(driver: &Driver, app: &mut TestServerRuntime) {
    let (_service, key) = support::service_key(driver);

    // Empty query uses defaults.
    // 200 OK response.
    let uri = format!("/v1/key");
    let (status_code, content_length, bytes) = support::app_get(app, &uri, Some(&key.value));
    assert_eq!(status_code, StatusCode::OK);
    assert_eq!(content_length, bytes.len());

    let body: server::route::key::ListResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.meta.gt, Some(0));
    assert_eq!(body.meta.lt, None);
    assert_eq!(body.meta.limit, 10);
    assert_eq!(body.data.len(), 1);

    let body_key = &body.data[0];
    assert!(body_key.created_at.eq(&key.created_at));
    assert!(body_key.updated_at.eq(&key.updated_at));
    assert_eq!(body_key.id, key.id);
    assert_eq!(body_key.name, key.name);
    assert_eq!(body_key.value, key.value);
    assert_eq!(body_key.service_id, key.service_id);
    assert_eq!(body_key.user_id, key.user_id);
}

pub fn create_authorisation_test(_driver: &Driver, app: &mut TestServerRuntime) {
    let payload = r#"{ "name": "test", "user_id": 1 }"#.as_bytes();
    support::post_authorisation_test(app, "/v1/key", payload)
}

pub fn read_authorisation_test(_driver: &Driver, app: &mut TestServerRuntime) {
    support::get_authorisation_test(app, "/v1/key/1")
}

// TODO(test): Key tests.
