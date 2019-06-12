use ark_auth::client::{Client, ClientOptions};
use ark_auth::core::{Key, Service, User};
use ark_auth::server::route::{auth, key, user};
use futures::Future;

pub fn server_url(uri: &str) -> String {
    let test_url = std::env::var("TEST_URL").unwrap();
    format!("{}{}", test_url, uri)
}

pub fn server_url2() -> String {
    std::env::var("TEST_URL").unwrap()
}

pub fn root_key() -> String {
    std::env::var("TEST_KEY").unwrap()
}

pub fn service_key_create2() -> (Service, Key) {
    let url = server_url2();
    let options = ClientOptions::new(&url, "test".to_owned(), root_key());
    let client = Client::new(options);

    let create = client
        .service_create("test".to_owned(), "http://localhost".to_owned())
        .wait()
        .unwrap();
    let service = create.data;

    let create = client
        .key_create("test".to_owned(), Some(service.id), None)
        .wait()
        .unwrap();
    let key = create.data;

    (service, key)
}

pub fn service_key_create(client: &reqwest::Client) -> (Service, Key) {
    let url = server_url("/v1/service");
    let request = ark_auth::server::route::service::CreateBody {
        name: "test".to_owned(),
        url: "http://localhost".to_owned(),
    };
    let mut response = client
        .post(&url)
        .header("content-type", "application/json")
        .header("authorization", root_key())
        .json(&request)
        .send()
        .unwrap();
    let response = response
        .json::<ark_auth::server::route::service::CreateResponse>()
        .unwrap();
    let service = response.data;

    let url = server_url("/v1/key");
    let request = ark_auth::server::route::key::CreateBody {
        name: "test".to_owned(),
        service_id: Some(service.id),
        user_id: None,
    };
    let mut response = client
        .post(&url)
        .header("content-type", "application/json")
        .header("authorization", root_key())
        .json(&request)
        .send()
        .unwrap();
    let response = response
        .json::<ark_auth::server::route::key::CreateResponse>()
        .unwrap();
    let key = response.data;

    (service, key)
}

pub fn user_email_create() -> String {
    let random = uuid::Uuid::new_v4().to_simple().to_string();
    format!("{}@example.com", random)
}

pub fn json_value(src: &str) -> serde_json::Value {
    serde_json::from_str(src).unwrap()
}

pub fn header_get<'a>(response: &'a reqwest::Response, name: &str) -> &'a str {
    response.headers().get(name).unwrap().to_str().unwrap()
}

pub fn user_post_200(
    service_key: &Key,
    name: &str,
    email: &str,
    active: bool,
    password: Option<&str>,
) -> User {
    let client = reqwest::Client::new();
    let request = user::CreateBody {
        name: name.to_owned(),
        email: email.to_owned(),
        active,
        password: password.map(String::from),
    };
    let url = server_url("/v1/user");
    let mut response = client
        .post(&url)
        .header("content-type", "application/json")
        .header("authorization", service_key.value.clone())
        .json(&request)
        .send()
        .unwrap();
    let body = response.json::<user::CreateResponse>().unwrap();
    let user = body.data;
    let status = response.status();
    let content_type = header_get(&response, "content-type");
    assert_eq!(status, 200);
    assert_eq!(content_type, "application/json");
    assert!(user.id > 0);
    assert_eq!(user.name, "User Name");
    assert_eq!(user.email, email);
    assert_eq!(user.active, active);
    assert!(user.password_hash.is_none());
    assert!(user.password_revision.is_none());
    user
}

pub fn key_post_user_200(service: &Service, service_key: &Key, user: &User, name: &str) -> Key {
    let client = reqwest::Client::new();
    let request = key::CreateBody {
        name: name.to_owned(),
        service_id: None,
        user_id: Some(user.id),
    };
    let url = server_url("/v1/key");
    let mut response = client
        .post(&url)
        .header("content-type", "application/json")
        .header("authorization", service_key.value.clone())
        .json(&request)
        .send()
        .unwrap();
    let body = response.json::<key::CreateResponse>().unwrap();
    let user_key = body.data;
    let status = response.status();
    let content_type = header_get(&response, "content-type");
    assert_eq!(status, 200);
    assert_eq!(content_type, "application/json");
    assert_eq!(user_key.name, "Key Name");
    assert_eq!(user_key.service_id.unwrap(), service.id);
    assert_eq!(user_key.user_id.unwrap(), user.id);
    user_key
}

pub fn auth_login_post_400(service_key: &Key, email: &str, password: &str) -> () {
    let client = reqwest::Client::new();
    let request = auth::LoginBody {
        email: email.to_owned(),
        password: password.to_owned(),
    };
    let url = server_url("/v1/auth/login");
    let response = client
        .post(&url)
        .header("content-type", "application/json")
        .header("authorization", service_key.value.clone())
        .json(&request)
        .send()
        .unwrap();
    let status = response.status();
    let content_length = header_get(&response, "content-length");
    assert_eq!(status, 400);
    assert_eq!(content_length, "0");
}
