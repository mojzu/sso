use ark_auth::client::{Client, ClientOptions};
use ark_auth::core::{Key, Service, User, UserKey};
use futures::Future;

pub fn env_test_url() -> String {
    std::env::var("TEST_URL").unwrap()
}

pub fn env_test_key() -> String {
    std::env::var("TEST_KEY").unwrap()
}

pub fn block_on_lazy<F, I, E>(fut: F) -> Result<I, E>
where
    F: Future<Item = I, Error = E>,
{
    actix_web::test::block_on(futures::future::lazy(|| fut))
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
    let create = block_on_lazy(client.service_create("test", "http://localhost")).unwrap();
    let service = create.data;

    let create = block_on_lazy(client.key_create("test", Some(service.id), None)).unwrap();
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
    let create = block_on_lazy(client.user_create(name, email, active, password)).unwrap();
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
    let create = block_on_lazy(client.key_create(name, None, Some(user_id))).unwrap();
    let key = create.data;
    assert_eq!(key.name, "Key Name");
    assert!(key.service_id.is_none());
    assert_eq!(key.user_id.unwrap(), user_id);
    key
}

pub fn verify_user_key(client: &Client, user_id: i64, key: &str) -> UserKey {
    let verify = block_on_lazy(client.auth_key_verify(key)).unwrap();
    let user_key = verify.data;
    assert_eq!(user_key.user_id, user_id);
    assert_eq!(user_key.key, key);
    user_key
}

pub fn request_password_reset(client: &Client, email: &str) -> () {
    block_on_lazy(client.auth_local_reset_password(email)).unwrap()
}
