mod audit;
mod auth_csrf;
mod auth_key;
mod auth_local;
mod auth_token;
mod auth_totp;
mod guide;
mod key;
mod service;
mod user;

pub use chrono::Utc;
pub use http::StatusCode;
pub use serde_json::Value;
pub use sso::{
    grpc::{pb, ClientBlocking, ClientOptions},
    AuditType, KeyType, KeyWithValue, Service, User, UserKey, UserToken, UserTokenAccess,
};
pub use uuid::Uuid;

pub const INVALID_EMAIL: &str = "invalid-email";
pub const INVALID_PASSWORD: &str = "guests";
pub const INVALID_KEY: &str = "af8731c10c739d8cce50ea556d0b1d77d3614fdc39";
pub const USER_NAME: &str = "user-name";
pub const USER_PASSWORD: &str = "user-name";
pub const KEY_NAME: &str = "key-name";

fn env_test_sso_url() -> String {
    std::env::var("TEST_SSO_GRPC_URL")
        .expect("TEST_SSO_GRPC_URL is undefined, integration test disabled")
}

fn env_test_sso_key() -> String {
    std::env::var("TEST_SSO_GRPC_KEY")
        .expect("TEST_SSO_GRPC_KEY is undefined, integration test disabled")
}

// #[test]
// #[ignore]
// fn test_ping() {
//     let mut client =
//         ClientBlocking::connect(&ClientOptions::new(env_test_sso_url()).authorisation(env_test_sso_key()))
//             .unwrap();
//     let request = tonic::Request::new(());
//     let response = client.ping(request).unwrap();
//     println!("RESPONSE={:?}", response);
// }

pub fn client_create(key: Option<&str>) -> ClientBlocking {
    match key {
        Some(key) => {
            ClientBlocking::connect(&ClientOptions::new(env_test_sso_url()).authorisation(key))
                .unwrap()
        }
        None => ClientBlocking::connect(
            &ClientOptions::new(env_test_sso_url()).authorisation(env_test_sso_key()),
        )
        .unwrap(),
    }
}

pub fn email_create() -> String {
    let random = Uuid::new_v4().to_simple().to_string();
    format!("{}@test.com", random)
}

pub fn service_key_create(client: &mut ClientBlocking) -> (pb::Service, pb::KeyWithValue) {
    let body = pb::ServiceCreateRequest::new(true, "test", "http://localhost")
        .provider_local_url("http://localhost")
        .provider_github_oauth2_url("http://localhost")
        .provider_microsoft_oauth2_url("http://localhost");
    let create_service = client
        .service_create(body)
        .unwrap()
        .into_inner()
        .data
        .unwrap();

    let body = pb::KeyCreateRequest::with_service_id(
        true,
        KeyType::Key,
        "test",
        create_service.id.clone(),
    );
    let create_key = client.key_create(body).unwrap().into_inner().data.unwrap();
    (create_service, create_key)
}

pub fn user_create(
    client: &mut ClientBlocking,
    is_enabled: bool,
    name: &str,
    email: &str,
) -> pb::User {
    // let before = Utc::now();
    let body = pb::UserCreateRequest::new(is_enabled, name, email);
    let user = client.user_create(body).unwrap().into_inner().data.unwrap();
    // assert!(user.created_at.gt(&before));
    // assert!(user.updated_at.gt(&before));
    // assert!(!user.id.is_nil());
    assert_eq!(user.is_enabled, is_enabled);
    assert_eq!(user.name, name);
    assert_eq!(user.email, email);
    user
}

pub fn user_create_with_password(
    client: &mut ClientBlocking,
    is_enabled: bool,
    name: &str,
    email: &str,
    password_allow_reset: bool,
    password_require_update: bool,
    password: &str,
) -> pb::User {
    // let before = Utc::now();
    let body = pb::UserCreateRequest::new(is_enabled, name, email).with_password(
        password_allow_reset,
        password_require_update,
        password,
    );
    let user = client.user_create(body).unwrap().into_inner().data.unwrap();
    // assert!(user.created_at.gt(&before));
    // assert!(user.updated_at.gt(&before));
    // assert!(!user.id.is_nil());
    assert_eq!(user.is_enabled, is_enabled);
    assert_eq!(user.name, name);
    assert_eq!(user.email, email);
    user
}

pub fn user_key_create(
    client: &mut ClientBlocking,
    name: &str,
    type_: KeyType,
    service_id: String,
    user: pb::User,
) -> (pb::User, pb::KeyWithValue) {
    let body = pb::KeyCreateRequest::with_user_id(true, type_, name, user.id.clone());
    let key = client.key_create(body).unwrap().into_inner().data.unwrap();
    let data = key.key.clone().unwrap();
    assert_eq!(data.name, name);
    assert_eq!(data.service_id.unwrap(), service_id);
    assert_eq!(data.user_id.unwrap(), user.id);
    (user, key)
}

pub fn user_key_verify(client: &mut ClientBlocking, key: &UserKey) -> pb::Key {
    let body = pb::AuthKeyRequest::new(&key.key, None);
    let verify = client.auth_key_verify(body).unwrap().into_inner();
    let user = verify.user.unwrap();
    let key = verify.key.unwrap();
    assert_eq!(user.id, key.user_id.clone().unwrap());
    key
}

// pub fn user_key_verify_bad_request(client: &ClientBlocking, key: &str) {
//     let body = pb::AuthKeyRequest::new(key, None);
//     let err = client.auth_key_verify(body).unwrap_err();
//     assert_eq!(err.status_code(), StatusCode::BAD_REQUEST.as_u16());
// }

// pub fn user_token_verify(client: &ClientBlocking, token: &UserToken) -> UserTokenAccess {
//     let body = pb::AuthTokenRequest::new(&token.access_token, None);
//     let verify = client.auth_token_verify(body).unwrap().into_inner();
//     let user = verify.user.unwrap();
//     let token = verify.access.unwrap();
//     assert_eq!(user.id, token.user.id);
//     assert_eq!(token.token, token.access_token);
//     assert_eq!(token.token_expires, token.access_token_expires);
//     user_token
// }

// pub fn user_token_refresh(client: &ClientBlocking, token: &UserToken) -> UserToken {
//     std::thread::sleep(std::time::Duration::from_secs(1));
//     let body = pb::AuthTokenRequest::new(&token.refresh_token, None);
//     let refresh = client.auth_token_refresh(body).unwrap().into_inner();
//     let user_token = refresh.data;
//     assert_eq!(user_token.user.id, token.user.id);
//     assert_ne!(user_token.access_token, token.access_token);
//     assert_ne!(user_token.access_token_expires, token.access_token_expires);
//     assert_ne!(user_token.access_token, token.access_token);
//     assert_ne!(user_token.access_token_expires, token.access_token_expires);
//     user_token
// }

// pub fn auth_local_login(
//     client: &ClientBlocking,
//     user_id: Uuid,
//     email: &str,
//     password: &str,
// ) -> UserToken {
//     let body = pb::AuthLoginRequest::new(email, password);
//     let login = client.auth_local_login(body).unwrap();
//     let user_token = login.data;
//     assert_eq!(user_token.user.id, user_id);
//     user_token
// }

// pub fn auth_microsoft_oauth2_url(client: &ClientBlocking) -> AuthOauth2UrlResponse {
//     let response = client.auth_microsoft_oauth2_url().unwrap();
//     assert!(!response.url.is_empty());
//     response
// }
