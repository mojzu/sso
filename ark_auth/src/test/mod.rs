use crate::{driver, server};
use actix_http::HttpService;
use actix_http_test::TestServer;
use actix_web::{web, App, HttpResponse};

pub fn app(
    driver: Box<driver::Driver>,
) -> (Box<driver::Driver>, actix_http_test::TestServerRuntime) {
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

#[macro_export]
macro_rules! integration_test {
    ($driver:expr) => {
        #[test]
        fn key_authorisation_test() {
            let (_, mut app) = $crate::test::app($driver);
            // TODO(refactor): Refactor integration tests here.
        }
    };
}
