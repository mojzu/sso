use sso_grpc::{pb::Empty, ClientBlocking, ClientOptions};

#[test]
#[ignore]
fn test_ping() {
    let mut client = ClientBlocking::connect(&ClientOptions::new("http://0.0.0.0:7000").authorisation("")).unwrap();
    let request = tonic::Request::new(Empty {});
    let response = client.ping(request).unwrap();
    println!("RESPONSE={:?}", response);
}
