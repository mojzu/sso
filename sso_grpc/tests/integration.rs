use sso_grpc::{pb::Empty, ClientBlocking};

#[test]
#[ignore]
fn test_ping() {
    let mut client = ClientBlocking::connect("http://0.0.0.0:7000").unwrap();
    let request = tonic::Request::new(Empty {});
    let response = client.ping(request).unwrap();
    println!("RESPONSE={:?}", response);
}
