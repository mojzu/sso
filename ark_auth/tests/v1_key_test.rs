mod support;

use actix_web::http::StatusCode;
use ark_auth::api;

#[test]
fn get_many_key_authorisation_test() {
    let (_db, mut app) = support::app();
    support::get_authorisation_test(&mut app, "/v1/key")
}

#[test]
fn get_many_key_query_test() {
    let (db, mut app) = support::app();
    let (_service, key) = support::service_key(&db);

    // Invalid query (invalid gt).
    // 400 BAD REQUEST response.
    let uri = format!("/v1/key?gt=-1");
    let (status_code, content_length, bytes) =
        support::app_get(&mut app, &uri, Some(&key.key_value));
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid query (invalid lt).
    // 400 BAD REQUEST response.
    let uri = format!("/v1/key?lt=-1");
    let (status_code, content_length, bytes) =
        support::app_get(&mut app, &uri, Some(&key.key_value));
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid query (invalid limit).
    // 400 BAD REQUEST response.
    let uri = format!("/v1/key?limit=-1");
    let (status_code, content_length, bytes) =
        support::app_get(&mut app, &uri, Some(&key.key_value));
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

#[test]
fn get_many_key_test() {
    let (db, mut app) = support::app();
    let (_service, key) = support::service_key(&db);

    // Empty query uses defaults.
    // 200 OK response.
    let uri = format!("/v1/key");
    let (status_code, content_length, bytes) =
        support::app_get(&mut app, &uri, Some(&key.key_value));
    assert_eq!(status_code, StatusCode::OK);
    assert_eq!(content_length, bytes.len());

    let body: api::key::ListResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.meta.gt, Some(0));
    assert_eq!(body.meta.lt, None);
    assert_eq!(body.meta.limit, 100);
    assert_eq!(body.data.len(), 1);

    let body_key = &body.data[0];
    assert!(body_key.created_at.eq(&key.created_at));
    assert!(body_key.updated_at.eq(&key.updated_at));
    assert_eq!(body_key.id, key.key_id);
    assert_eq!(body_key.name, key.key_name);
    assert_eq!(body_key.value, key.key_value);
    assert_eq!(body_key.service_id, key.service_id);
    assert_eq!(body_key.user_id, key.user_id);
}

#[test]
fn post_key_authorisation_test() {
    let (_db, mut app) = support::app();
    let payload = r#"{ "name": "test", "user_id": 1 }"#.as_bytes();
    support::post_authorisation_test(&mut app, "/v1/key", payload)
}

// TODO(test)
