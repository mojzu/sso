#[macro_export]
macro_rules! auth_key_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_auth_key_verify_forbidden() {
            let mut client = client_create();

            client.options.set_authorisation(INVALID_SERVICE_KEY);
            let res = client.auth_key_verify(INVALID_UUID).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::Forbidden));
        }

        #[test]
        #[ignore]
        fn api_auth_key_verify_bad_request_invalid_key() {
            let mut client = client_create();
            let (_service, service_key) = service_key_create(&client);

            client.options.set_authorisation(&service_key.value);
            let res = client.auth_key_verify(INVALID_UUID).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_key_verify_bad_request_unknown_user_key_for_service() {
            let mut client = client_create();
            let (service1, service1_key) = service_key_create(&client);
            let (_service2, service2_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service1_key.value);
            let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
            let user_key = user_key_create(&client, KEY_NAME, &service1.id, &user.id);

            client.options.set_authorisation(&service2_key.value);
            let res = client.auth_key_verify(&user_key.key).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_key_verify_bad_request_service_key() {
            let mut client = client_create();
            let (_service, service_key) = service_key_create(&client);

            client.options.set_authorisation(&service_key.value);
            let res = client.auth_key_verify(&service_key.value).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_key_verify_ok() {
            let mut client = client_create();
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service_key.value);
            let user = user_create(&client, true, USER_NAME, &user_email, None);
            let user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

            client.auth_key_verify(&user_key.key).unwrap();
        }

        #[test]
        #[ignore]
        fn api_auth_key_revoke_forbidden() {
            let mut client = client_create();

            client.options.set_authorisation(INVALID_SERVICE_KEY);
            let res = client.auth_key_revoke(INVALID_UUID).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::Forbidden));
        }

        #[test]
        #[ignore]
        fn api_auth_key_revoke_bad_request_invalid_key() {
            let mut client = client_create();
            let (_service, service_key) = service_key_create(&client);

            client.options.set_authorisation(&service_key.value);
            let res = client.auth_key_revoke(INVALID_UUID).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_key_revoke_bad_request_unknown_user_key_for_service() {
            let mut client = client_create();
            let (service1, service1_key) = service_key_create(&client);
            let (_service2, service2_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service1_key.value);
            let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
            let user_key = user_key_create(&client, KEY_NAME, &service1.id, &user.id);

            client.options.set_authorisation(&service2_key.value);
            let res = client.auth_key_revoke(&user_key.key).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_key_revoke_bad_request_service_key() {
            let mut client = client_create();
            let (_service, service_key) = service_key_create(&client);

            client.options.set_authorisation(&service_key.value);
            let res = client.auth_key_revoke(&service_key.value).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_key_revoke_ok() {
            let mut client = client_create();
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service_key.value);
            let user = user_create(&client, true, USER_NAME, &user_email, None);
            let user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

            client.auth_key_revoke(&user_key.key).unwrap();
            let res = client.auth_key_verify(&user_key.key).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }
    };
}
