#[macro_export]
macro_rules! auth_csrf_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_auth_csrf_create_unauthorised() {
            let client = client_create(Some(INVALID_KEY));
            let body = AuthCsrfCreateRequest::new(500);
            let res = client.auth_csrf_create(body).unwrap_err();
            assert_eq!(res, ClientError::Unauthorised);
        }

        // TODO(test): CSRF endpoint tests.
    };
}
