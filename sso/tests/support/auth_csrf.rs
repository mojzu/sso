#[macro_export]
macro_rules! auth_csrf_integration_test {
    () => {
        #[test]
        #[ignore]
        fn auth_csrf_create_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let body = pb::AuthCsrfCreateRequest::new(500);
            let res = client.auth_csrf_create(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_csrf_bad_request_verify_once() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthCsrfCreateRequest::new(500);
            let res = client.auth_csrf_create(body).unwrap().into_inner();
            let csrf = res.csrf.unwrap();

            let body = pb::AuthCsrfVerifyRequest::new(&csrf.key);
            client.auth_csrf_verify(body).unwrap();
            let body = pb::AuthCsrfVerifyRequest::new(&csrf.key);
            client.auth_csrf_verify(body).unwrap_err();
        }

        #[test]
        #[ignore]
        fn auth_csrf_create_verify_ok() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthCsrfCreateRequest::new(500);
            let res = client.auth_csrf_create(body).unwrap().into_inner();
            let csrf = res.csrf.unwrap();

            let body = pb::AuthCsrfVerifyRequest::new(&csrf.key);
            client.auth_csrf_verify(body).unwrap();
        }
    };
}
