mod support;

use ark_auth::client::{Error, RequestError};
use support::*;
use serde_json::Value;

#[test]
fn guide_api_key() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, "user", &user_email, None);
    let user_key = user_key_create(&client, "key", &service.id, &user.id);

    user_key_verify(&client, &user_key);
    client.auth_key_revoke(&user_key.key).unwrap();
    user_key_verify_bad_request(&client, &user_key.key);
}

#[test]
fn guide_login() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, "user", &user_email, Some("guest"));
    let _user_key = user_key_create(&client, "key", &service.id, &user.id);

    let user_token = auth_local_login(&client, &user.id, &user_email, "guest");
    user_token_verify(&client, &user_token);
    let user_token = user_token_refresh(&client, &user_token);
    client.auth_token_revoke(&user_token.access_token).unwrap();
    user_token_verify_bad_request(&client, &user_token.refresh_token);
}

#[test]
fn guide_reset_password() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, "user", &user_email, Some("guest"));
    let _user_key = user_key_create(&client, "key", &service.id, &user.id);

    client.auth_local_reset_password(&user_email).unwrap();
}

#[test]
fn guide_oauth2_login() {
    let mut client = client_create();
    let (service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    let user = user_create(&client, true, "user", &user_email, Some("guest"));
    let _user_key = user_key_create(&client, "key", &service.id, &user.id);

    auth_microsoft_oauth2_request(&client);
}

#[test]
fn api_ping_ok() {
    let client = client_create();
    let pong = client.ping().unwrap();
    assert_eq!(pong, Value::String("pong".to_owned()));
}

#[test]
fn api_user_create_ok() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    user_create(&client, true, "user", &user_email, None);
}

#[test]
fn api_user_create_forbidden() {
    let mut client = client_create();
    let user_email = email_create();

    client.options.set_authorisation("invalid-service-key");
    let create = client
        .user_create(true, "user", &user_email, None)
        .unwrap_err();
    assert_eq!(create, Error::Request(RequestError::Forbidden));
}

#[test]
fn api_user_create_bad_request_duplicate_user_email() {
    let mut client = client_create();
    let (_service, service_key) = service_key_create(&client);
    let user_email = email_create();

    client.options.set_authorisation(&service_key.value);
    user_create(&client, true, "user", &user_email, None);

    let create = client
        .user_create(true, "user", &user_email, None)
        .unwrap_err();
    assert_eq!(create, Error::Request(RequestError::BadRequest));
}

#[test]
fn api_audit_id_list_ok() {
    let client = client_create();
    let data = Value::Null;
    client.audit_create("/test/1", &data, None, None).unwrap();
    client.audit_create("/test/2", &data, None, None).unwrap();
    client.audit_create("/test/3", &data, None, None).unwrap();
    client.audit_create("/test/4", &data, None, None).unwrap();
    client.audit_create("/test/5", &data, None, None).unwrap();

    let res1 = client.audit_list(None, None, Some(3)).unwrap();
    assert_eq!(res1.data.len(), 3);
    let r1_1 = &res1.data[0];
    let r1_2 = &res1.data[1];
    let r1_3 = &res1.data[2];

    let res2 = client.audit_list(Some(r1_1), None, Some(3)).unwrap();
    assert_eq!(res2.data.len(), 3);
    let r2_2 = &res2.data[0];
    let r2_3 = &res2.data[1];
    let r2_4 = &res2.data[2];
    assert_eq!(r2_2, r1_2);
    assert_eq!(r2_3, r1_3);

    let res3 = client.audit_list(Some(r1_2), None, Some(3)).unwrap();
    assert_eq!(res3.data.len(), 3);
    let r3_3 = &res3.data[0];
    let r3_4 = &res3.data[1];
    let r3_5 = &res3.data[2];
    assert_eq!(r3_3, r2_3);
    assert_eq!(r3_4, r2_4);

    let res4 = client.audit_list(None, Some(r3_5), Some(3)).unwrap();
    assert_eq!(res4.data.len(), 3);
    let r4_2 = &res4.data[0];
    let r4_3 = &res4.data[1];
    let r4_4 = &res4.data[2];
    assert_eq!(r4_2, r2_2);
    assert_eq!(r4_3, r3_3);
    assert_eq!(r4_4, r3_4);

    let res5 = client.audit_list(None, Some(r4_4), Some(3)).unwrap();
    assert_eq!(res5.data.len(), 3);
    let r5_1 = &res5.data[0];
    let r5_2 = &res5.data[1];
    let r5_3 = &res5.data[2];
    assert_eq!(r5_1, r1_1);
    assert_eq!(r5_2, r4_2);
    assert_eq!(r5_3, r4_3);
}
