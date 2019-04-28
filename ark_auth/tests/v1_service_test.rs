mod support;

#[test]
fn get_many_service_authorisation_test() {
    let (_db, mut app) = support::app();
    support::get_authorisation_test(&mut app, "/v1/service")
}

#[test]
fn post_service_authorisation_test() {
    let (_db, mut app) = support::app();
    let payload = r#"{ "name": "test", "url": "localhost:9001" }"#.as_bytes();
    support::post_authorisation_test(&mut app, "/v1/service", payload)
}

// TODO(test)
