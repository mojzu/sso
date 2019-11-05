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
    api::{
        AuditCreateRequestBuilder, AuditListRequestBuilder, AuditReadRequest, AuditUpdateRequest,
        AuthCsrfCreateRequest, AuthCsrfVerifyRequest, AuthKeyRequest, AuthLoginRequest,
        AuthOauth2UrlResponse, AuthRegisterRequest, AuthResetPasswordConfirmRequest,
        AuthResetPasswordRequest, AuthTokenRequest, AuthTotpRequest, AuthUpdateEmailRequest,
        KeyCreateRequest, KeyListRequestBuilder, KeyReadRequest, ServiceCreateRequest,
        ServiceListRequestBuilder, ServiceUpdateRequest, UserCreateRequest, UserListRequestBuilder,
        UserUpdateRequest,
    },
    AuditType, ClientOptions, ClientRequestOptions, ClientSync, KeyType, KeyWithValue, Service,
    User, UserKey, UserToken, UserTokenAccess,
};
pub use uuid::Uuid;

pub const INVALID_EMAIL: &str = "invalid-email";
pub const INVALID_PASSWORD: &str = "guests";
pub const INVALID_KEY: &str = "af8731c10c739d8cce50ea556d0b1d77d3614fdc39";
pub const USER_NAME: &str = "user-name";
pub const USER_PASSWORD: &str = "user-name";
pub const KEY_NAME: &str = "key-name";

lazy_static! {
    static ref CLIENT: ClientSync = {
        let authorisation = env_test_sso_key();
        let options =
            ClientOptions::new("test", None, None, ClientRequestOptions::new(authorisation))
                .unwrap();
        let url = env_test_sso_url();
        ClientSync::new(url, options).unwrap()
    };
}

fn env_test_sso_url() -> String {
    std::env::var("TEST_SSO_URL").expect("TEST_SSO_URL is undefined, integration test disabled")
}

fn env_test_sso_key() -> String {
    std::env::var("TEST_SSO_KEY").expect("TEST_SSO_KEY is undefined, integration test disabled")
}

pub fn client_create(key: Option<&str>) -> ClientSync {
    match key {
        Some(key) => CLIENT.with_options(ClientRequestOptions::new(key.to_owned())),
        None => CLIENT.clone(),
    }
}

pub fn email_create() -> String {
    let random = Uuid::new_v4().to_simple().to_string();
    format!("{}@test.com", random)
}

pub fn service_key_create(client: &ClientSync) -> (Service, KeyWithValue) {
    let body = ServiceCreateRequest::new(true, "test", "http://localhost")
        .provider_local_url("http://localhost")
        .provider_github_oauth2_url("http://localhost")
        .provider_microsoft_oauth2_url("http://localhost");
    let create_service = client.service_create(body).unwrap();

    let body =
        KeyCreateRequest::with_service_id(true, KeyType::Key, "test", create_service.data.id);
    let create_key = client.key_create(body).unwrap();
    (create_service.data, create_key.data)
}

pub fn user_create(client: &ClientSync, is_enabled: bool, name: &str, email: &str) -> User {
    let before = Utc::now();
    let body = UserCreateRequest::new(is_enabled, name, email);
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
    password_allow_reset: bool,
    password_require_update: bool,
    password: &str,
) -> User {
    let before = Utc::now();
    let body = UserCreateRequest::new(is_enabled, name, email).with_password(
        password_allow_reset,
        password_require_update,
        password,
    );
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
    type_: KeyType,
    service_id: Uuid,
    user: User,
) -> UserKey {
    let body = KeyCreateRequest::with_user_id(true, type_, name, user.id);
    let create = client.key_create(body).unwrap();
    let key = create.data;
    assert_eq!(key.name, name);
    assert_eq!(key.service_id.unwrap(), service_id);
    assert_eq!(key.user_id.unwrap(), user.id);
    UserKey {
        user,
        key: key.value.to_owned(),
    }
}

pub fn user_key_create2(
    client: &ClientSync,
    name: &str,
    type_: KeyType,
    service_id: Uuid,
    user: User,
) -> (User, KeyWithValue) {
    let body = KeyCreateRequest::with_user_id(true, type_, name, user.id);
    let create = client.key_create(body).unwrap();
    let key = create.data;
    assert_eq!(key.name, name);
    assert_eq!(key.service_id.unwrap(), service_id);
    assert_eq!(key.user_id.unwrap(), user.id);
    (user, key)
}

pub fn user_key_verify(client: &ClientSync, key: &UserKey) -> UserKey {
    let body = AuthKeyRequest::new(&key.key, None);
    let verify = client.auth_key_verify(body).unwrap();
    let user_key = verify.data;
    assert_eq!(user_key.user.id, key.user.id);
    assert_eq!(user_key.key, key.key);
    user_key
}

pub fn user_key_verify_bad_request(client: &ClientSync, key: &str) {
    let body = AuthKeyRequest::new(key, None);
    let err = client.auth_key_verify(body).unwrap_err();
    assert_eq!(err.status_code(), StatusCode::BAD_REQUEST.as_u16());
}

pub fn user_token_verify(client: &ClientSync, token: &UserToken) -> UserTokenAccess {
    let body = AuthTokenRequest::new(&token.access_token, None);
    let verify = client.auth_token_verify(body).unwrap();
    let user_token = verify.data;
    assert_eq!(user_token.user.id, token.user.id);
    assert_eq!(user_token.access_token, token.access_token);
    assert_eq!(user_token.access_token_expires, token.access_token_expires);
    user_token
}

pub fn user_token_refresh(client: &ClientSync, token: &UserToken) -> UserToken {
    std::thread::sleep(std::time::Duration::from_secs(1));
    let body = AuthTokenRequest::new(&token.refresh_token, None);
    let refresh = client.auth_token_refresh(body).unwrap();
    let user_token = refresh.data;
    assert_eq!(user_token.user.id, token.user.id);
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
    let body = AuthLoginRequest::new(email, password);
    let login = client.auth_local_login(body).unwrap();
    let user_token = login.data;
    assert_eq!(user_token.user.id, user_id);
    user_token
}

pub fn auth_microsoft_oauth2_url(client: &ClientSync) -> AuthOauth2UrlResponse {
    let response = client.auth_microsoft_oauth2_url().unwrap();
    assert!(!response.url.is_empty());
    response
}
