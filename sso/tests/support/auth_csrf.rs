#[macro_export]
macro_rules! auth_csrf_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_auth_csrf_create_unauthorised() {
            let client = client_create(Some(INVALID_KEY));
            let body = AuthCsrfCreateRequest::new(500);
            let res = client.auth_csrf_create(body).unwrap_err();
            assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED.as_u16());
        }

        #[test]
        #[ignore]
        fn api_auth_csrf_bad_request_verify_once() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let body = AuthCsrfCreateRequest::new(500);
            let res = client.auth_csrf_create(body).unwrap();
            let csrf = res.data;

            let body = AuthCsrfVerifyRequest::new(&csrf.key);
            client.auth_csrf_verify(body).unwrap();
            let body = AuthCsrfVerifyRequest::new(&csrf.key);
            client.auth_csrf_verify(body).unwrap_err();
        }

        #[test]
        #[ignore]
        fn api_auth_csrf_create_verify_ok() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let body = AuthCsrfCreateRequest::new(500);
            let res = client.auth_csrf_create(body).unwrap();
            let csrf = res.data;

            let body = AuthCsrfVerifyRequest::new(&csrf.key);
            client.auth_csrf_verify(body).unwrap();
        }
    };
}
