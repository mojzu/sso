mod support;

#[test]
fn get_many_key_authorisation_test() {
    let (_db, mut app) = support::app();
    support::get_authorisation_test(&mut app, "/v1/key")
}

#[test]
fn post_key_authorisation_test() {
    let (_db, mut app) = support::app();
    let payload = r#"{ "name": "test", "user_id": 1 }"#.as_bytes();
    support::post_authorisation_test(&mut app, "/v1/key", payload)
}

// TODO(test)
