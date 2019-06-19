use ark_auth::client::{Client, ClientOptions, Error, RequestError, SyncClient};
use ark_auth::core::{Key, Service, User, UserKey, UserToken};
use ark_auth::server::route::auth::provider::Oauth2UrlResponse;
use chrono::Utc;

pub fn env_test_url() -> String {
    std::env::var("TEST_URL").unwrap()
}

pub fn env_test_key() -> String {
    std::env::var("TEST_KEY").unwrap()
}

pub fn create_user_email() -> String {
    let random = uuid::Uuid::new_v4().to_simple().to_string();
    format!("{}@example.com", random)
}

pub fn create_client() -> SyncClient {
    let url = env_test_url();
    let key = env_test_key();
    let options = ClientOptions::new(&url, &key).unwrap();
    SyncClient::new(options)
}

pub fn ping_server(client: &SyncClient) {
    let pong = client.ping().unwrap();
    assert_eq!(pong, serde_json::Value::String("pong".to_owned()));
}

pub fn create_service_key(client: &SyncClient) -> (Service, Key) {
    let create_service = client.service_create("test", "http://localhost").unwrap();
    let create_key = client
        .key_create("test", Some(create_service.data.id), None)
        .unwrap();
    (create_service.data, create_key.data)
}

pub fn create_user(
    client: &SyncClient,
    name: &str,
    email: &str,
    active: bool,
    password: Option<&str>,
) -> User {
    let before = Utc::now();
    let create = client.user_create(name, email, active, password).unwrap();
    let user = create.data;
    assert!(user.created_at.gt(&before));
    assert!(user.updated_at.gt(&before));
    assert!(user.id > 0);
    assert_eq!(user.name, name);
    assert_eq!(user.email, email);
    assert_eq!(user.active, active);
    assert!(user.password_hash.is_none());
    assert!(user.password_revision.is_none());
    user
}

pub fn create_user_duplicate_email(
    client: &SyncClient,
    name: &str,
    email: &str,
    active: bool,
    password: Option<&str>,
) {
    let create = client
        .user_create(name, email, active, password)
        .unwrap_err();
    assert_eq!(create, Error::Request(RequestError::BadRequest));
}

pub fn create_user_forbidden(
    client: &SyncClient,
    name: &str,
    email: &str,
    active: bool,
    password: Option<&str>,
) {
    let create = client
        .user_create(name, email, active, password)
        .unwrap_err();
    assert_eq!(create, Error::Request(RequestError::Forbidden));
}

pub fn create_user_key(client: &SyncClient, name: &str, service_id: i64, user_id: i64) -> UserKey {
    let create = client.key_create(name, None, Some(user_id)).unwrap();
    let key = create.data;
    assert_eq!(key.name, "Key Name");
    assert_eq!(key.service_id.unwrap(), service_id);
    assert_eq!(key.user_id.unwrap(), user_id);
    UserKey {
        user_id,
        key: key.value.to_owned(),
    }
}

pub fn verify_user_key(client: &SyncClient, key: &UserKey) -> UserKey {
    let verify = client.auth_key_verify(key).unwrap();
    let user_key = verify.data;
    assert_eq!(user_key.user_id, key.user_id);
    assert_eq!(user_key.key, key.key);
    user_key
}

pub fn verify_user_token(client: &SyncClient, token: &UserToken) -> UserToken {
    let verify = client.auth_token_verify(token).unwrap();
    let user_token = verify.data;
    assert_eq!(user_token.user_id, token.user_id);
    assert_eq!(user_token.token, token.token);
    assert_eq!(user_token.token_expires, token.token_expires);
    user_token
}

pub fn local_login(client: &SyncClient, user_id: i64, email: &str, password: &str) -> UserToken {
    let login = client.auth_local_login(email, password).unwrap();
    let user_token = login.data;
    assert_eq!(user_token.user_id, user_id);
    user_token
}

pub fn local_password_reset(client: &SyncClient, email: &str) {
    client.auth_local_reset_password(email).unwrap()
}

pub fn microsoft_oauth2_request(client: &SyncClient) -> Oauth2UrlResponse {
    let response = client.auth_microsoft_oauth2_request().unwrap();
    assert!(!response.url.is_empty());
    response
}
