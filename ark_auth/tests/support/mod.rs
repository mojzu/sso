mod audit;
mod auth_key;
mod auth_local;
mod auth_token;
mod guide;
mod key;
mod service;
mod user;

pub use ark_auth::server_api::{
    AuditCreateBody, AuditListQuery, AuthKeyBody, AuthLoginBody, AuthResetPasswordBody,
    AuthResetPasswordConfirmBody, AuthTokenBody, KeyCreateBody, KeyListQuery, ServiceCreateBody,
    ServiceListQuery, UserCreateBody, UserListQuery,
};
use ark_auth::{
    server_api::AuthOauth2UrlResponse, Key, Service, User, UserAccessToken, UserKey, UserToken,
};
pub use ark_auth::{Client, ClientActorOptions, ClientError, ClientOptions, ClientSync};
use chrono::Utc;
pub use serde_json::Value;
pub use uuid::Uuid;

pub const INVALID_EMAIL: &str = "invalid-email";
pub const INVALID_PASSWORD: &str = "guests";
pub const INVALID_KEY: &str = "af8731c10c739d8cce50ea556d0b1d77d3614fdc39";
pub const USER_NAME: &str = "user-name";
pub const USER_PASSWORD: &str = "user-name";
pub const KEY_NAME: &str = "key-name";

lazy_static! {
    static ref CLIENT: ClientSync = {
        let executor_options =
            ClientActorOptions::new(Client::default_user_agent(), None, None).unwrap();
        let authorisation = env_test_ark_auth_key();
        let options = ClientOptions::new(authorisation);
        let url = env_test_ark_auth_url();
        ClientSync::new(url, executor_options, options).unwrap()
    };
}

fn env_test_ark_auth_url() -> String {
    std::env::var("TEST_ARK_AUTH_URL")
        .expect("TEST_ARK_AUTH_URL is undefined, integration test disabled")
}

fn env_test_ark_auth_key() -> String {
    std::env::var("TEST_ARK_AUTH_KEY")
        .expect("TEST_ARK_AUTH_KEY is undefined, integration test disabled")
}

pub fn client_create(key: Option<&str>) -> ClientSync {
    match key {
        Some(key) => CLIENT.with_options(ClientOptions::new(key.to_owned())),
        None => CLIENT.clone(),
    }
}

pub fn email_create() -> String {
    let random = Uuid::new_v4().to_simple().to_string();
    format!("{}@test.com", random)
}

pub fn service_key_create(client: &ClientSync) -> (Service, Key) {
    let body = ServiceCreateBody::new(true, "test", "http://localhost");
    let create_service = client.service_create(body).unwrap();

    let body = KeyCreateBody::with_service_id(true, "test", create_service.data.id);
    let create_key = client.key_create(body).unwrap();
    (create_service.data, create_key.data)
}

pub fn user_create(client: &ClientSync, is_enabled: bool, name: &str, email: &str) -> User {
    let before = Utc::now();
    let body = UserCreateBody::new(is_enabled, name, email);
    let create = client.user_create(body).unwrap();
    let user = create.data;
    assert!(user.created_at.gt(&before));
    assert!(user.updated_at.gt(&before));
    assert!(!user.id.is_nil());
    assert_eq!(user.is_enabled, is_enabled);
    assert_eq!(user.name, name);
    assert_eq!(user.email, email);
    assert!(user.password_hash.is_none());
    user
}

pub fn user_create_with_password(
    client: &ClientSync,
    is_enabled: bool,
    name: &str,
    email: &str,
    password: &str,
) -> User {
    let before = Utc::now();
    let body = UserCreateBody::with_password(is_enabled, name, email, password);
    let create = client.user_create(body).unwrap();
    let user = create.data;
    assert!(user.created_at.gt(&before));
    assert!(user.updated_at.gt(&before));
    assert!(!user.id.is_nil());
    assert_eq!(user.is_enabled, is_enabled);
    assert_eq!(user.name, name);
    assert_eq!(user.email, email);
    assert!(user.password_hash.is_none());
    user
}

pub fn user_key_create(
    client: &ClientSync,
    name: &str,
    service_id: Uuid,
    user_id: Uuid,
) -> UserKey {
    let body = KeyCreateBody::with_user_id(true, name, user_id);
    let create = client.key_create(body).unwrap();
    let key = create.data;
    assert_eq!(key.name, name);
    assert_eq!(key.service_id.unwrap(), service_id);
    assert_eq!(key.user_id.unwrap(), user_id);
    UserKey {
        user_id: user_id.to_owned(),
        key: key.value.to_owned(),
    }
}

pub fn user_key_verify(client: &ClientSync, key: &UserKey) -> UserKey {
    let body = AuthKeyBody::new(&key.key, None);
    let verify = client.auth_key_verify(body).unwrap();
    let user_key = verify.data;
    assert_eq!(user_key.user_id, key.user_id);
    assert_eq!(user_key.key, key.key);
    user_key
}

pub fn user_key_verify_bad_request(client: &ClientSync, key: &str) {
    let body = AuthKeyBody::new(key, None);
    let err = client.auth_key_verify(body).unwrap_err();
    assert_eq!(err, ClientError::BadRequest);
}

pub fn user_token_verify(client: &ClientSync, token: &UserToken) -> UserAccessToken {
    let body = AuthTokenBody::new(&token.access_token, None);
    let verify = client.auth_token_verify(body).unwrap();
    let user_token = verify.data;
    assert_eq!(user_token.user_id, token.user_id);
    assert_eq!(user_token.access_token, token.access_token);
    assert_eq!(user_token.access_token_expires, token.access_token_expires);
    user_token
}

pub fn user_token_refresh(client: &ClientSync, token: &UserToken) -> UserToken {
    std::thread::sleep(std::time::Duration::from_secs(1));
    let body = AuthTokenBody::new(&token.refresh_token, None);
    let refresh = client.auth_token_refresh(body).unwrap();
    let user_token = refresh.data;
    assert_eq!(user_token.user_id, token.user_id);
    assert_ne!(user_token.access_token, token.access_token);
    assert_ne!(user_token.access_token_expires, token.access_token_expires);
    assert_ne!(user_token.access_token, token.access_token);
    assert_ne!(user_token.access_token_expires, token.access_token_expires);
    user_token
}

pub fn auth_local_login(
    client: &ClientSync,
    user_id: Uuid,
    email: &str,
    password: &str,
) -> UserToken {
    let body = AuthLoginBody::new(email, password);
    let login = client.auth_local_login(body).unwrap();
    let user_token = login.data;
    assert_eq!(user_token.user_id, user_id);
    user_token
}

pub fn auth_microsoft_oauth2_request(client: &ClientSync) -> AuthOauth2UrlResponse {
    let response = client.auth_microsoft_oauth2_request().unwrap();
    assert!(!response.url.is_empty());
    response
}
