use ark_auth::client::{Client, ClientOptions};
use ark_auth::core::{Key, Service, User, UserKey};
use futures::Future;

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

pub fn create_client() -> Client {
    let url = env_test_url();
    let key = env_test_key();
    let options = ClientOptions::new(&url, "test", &key);
    Client::new(options)
}

pub fn create_service_key(client: &Client) -> (Service, Key) {
    let create = client
        .service_create("test", "http://localhost")
        .wait()
        .unwrap();
    let service = create.data;

    let create = client
        .key_create("test", Some(service.id), None)
        .wait()
        .unwrap();
    let key = create.data;

    (service, key)
}

pub fn create_user(
    client: &Client,
    name: &str,
    email: &str,
    active: bool,
    password: Option<&str>,
) -> User {
    let create = client
        .user_create(name, email, active, password)
        .wait()
        .unwrap();
    let user = create.data;
    assert!(user.id > 0);
    assert_eq!(user.name, name);
    assert_eq!(user.email, email);
    assert_eq!(user.active, active);
    assert!(user.password_hash.is_none());
    assert!(user.password_revision.is_none());
    user
}

pub fn create_user_key(client: &Client, name: &str, user_id: i64) -> Key {
    let create = client.key_create(name, None, Some(user_id)).wait().unwrap();
    let key = create.data;
    assert_eq!(key.name, "Key Name");
    assert!(key.service_id.is_none());
    assert_eq!(key.user_id.unwrap(), user_id);
    key
}

pub fn verify_user_key(client: &Client, user_id: i64, key: &str) -> UserKey {
    let verify = client.auth_key_verify(key).wait().unwrap();
    let user_key = verify.data;
    assert_eq!(user_key.user_id, user_id);
    assert_eq!(user_key.key, key);
    user_key
}

pub fn request_password_reset(client: &Client, email: &str) -> () {
    client.auth_reset_password(email).wait().unwrap();
}
