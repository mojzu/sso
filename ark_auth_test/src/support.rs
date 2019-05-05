use actix_http::HttpService;
use actix_http_test::{TestServer, TestServerRuntime};
use actix_web::{
    http::{header, StatusCode},
    web, App, HttpResponse,
};
use ark_auth::{driver, server};

pub fn app(driver: Box<driver::Driver>) -> (Box<driver::Driver>, TestServerRuntime) {
    let configuration = server::Configuration::new("localhost:9001".to_owned());
    let driver_clone = driver.clone();

    let server = TestServer::new(move || {
        HttpService::new(
            App::new()
                .data(server::Data::new(
                    configuration.clone(),
                    driver_clone.clone(),
                ))
                .wrap(server::AuthorisationIdentityPolicy::identity_service())
                .configure(server::api_service)
                .default_service(web::route().to(|| HttpResponse::MethodNotAllowed())),
        )
    });

    (driver, server)
}

pub fn app_post<T: Into<actix_http::body::Body>>(
    app: &mut TestServerRuntime,
    uri: &str,
    authorisation: Option<&str>,
    payload: T,
) -> (StatusCode, usize, Vec<u8>) {
    let req = app
        .post(uri)
        .header(header::CONTENT_TYPE, header::ContentType::json());
    let req = match authorisation {
        Some(authorisation) => req.header(header::AUTHORIZATION, authorisation),
        None => req,
    };
    let req = req.send_body(payload);
    let mut res = app.block_on(req).unwrap();

    let status_code = res.status();
    let content_length = res
        .headers()
        .get(header::CONTENT_LENGTH)
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let bytes = app.block_on(res.body()).unwrap();

    (status_code, content_length, bytes.to_vec())
}

pub fn app_get(
    app: &mut TestServerRuntime,
    uri: &str,
    authorisation: Option<&str>,
) -> (StatusCode, usize, Vec<u8>) {
    let req = app.get(uri);
    let req = match authorisation {
        Some(authorisation) => req.header(header::AUTHORIZATION, authorisation),
        None => req,
    };
    let req = req.send();
    let mut res = app.block_on(req).unwrap();

    let status_code = res.status();
    let content_length = res
        .headers()
        .get(header::CONTENT_LENGTH)
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let bytes = app.block_on(res.body()).unwrap();

    (status_code, content_length, bytes.to_vec())
}

pub fn post_authorisation_test<T: Into<actix_http::body::Body> + Clone>(
    app: &mut TestServerRuntime,
    uri: &str,
    payload: T,
) {
    // Missing header.
    // 403 FORBIDDEN response.
    let (status_code, content_length, bytes) = app_post(app, uri, None, payload.clone());
    assert_eq!(status_code, StatusCode::FORBIDDEN);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid header.
    // 403 FORBIDDEN response.
    let (status_code, content_length, bytes) = app_post(app, uri, Some("invalid"), payload.clone());
    assert_eq!(status_code, StatusCode::FORBIDDEN);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}

pub fn get_authorisation_test(app: &mut TestServerRuntime, uri: &str) {
    // Missing header.
    // 403 FORBIDDEN response.
    let (status_code, content_length, bytes) = app_get(app, uri, None);
    assert_eq!(status_code, StatusCode::FORBIDDEN);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);

    // Invalid header.
    // 403 FORBIDDEN response.
    let (status_code, content_length, bytes) = app_get(app, uri, Some("invalid"));
    assert_eq!(status_code, StatusCode::FORBIDDEN);
    assert_eq!(content_length, 0);
    assert_eq!(bytes.len(), 0);
}
