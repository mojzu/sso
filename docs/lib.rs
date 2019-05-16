use ark_auth::core::{Key, Service};

pub fn server_url(uri: &str) -> String {
    let test_url = std::env::var("TEST_URL").unwrap();
    format!("{}{}", test_url, uri)
}

pub fn root_key() -> String {
    std::env::var("TEST_KEY").unwrap()
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
