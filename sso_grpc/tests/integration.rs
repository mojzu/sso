use sso_grpc::{client::SsoClientBlocking, pb::Empty};

#[test]
fn test_ping() {
    let mut client = SsoClientBlocking::connect("http://0.0.0.0:9090").unwrap();

    let request = tonic::Request::new(Empty {});

    let response = client.ping(request).unwrap();

    println!("RESPONSE={:?}", response);
}
