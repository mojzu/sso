#[macro_export]
macro_rules! auth_key_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_auth_key_verify_unauthorised() {
            let client = client_create(Some(INVALID_KEY));
            let body = AuthKeyRequest::new(INVALID_KEY, None);
            let res = client.auth_key_verify(body).unwrap_err();
            assert_eq!(res, ClientError::Unauthorised);
        }

        #[test]
        #[ignore]
        fn api_auth_key_verify_bad_request_invalid_key() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let body = AuthKeyRequest::new(INVALID_KEY, None);
            let res = client.auth_key_verify(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_auth_key_verify_bad_request_unknown_user_key_for_service() {
            let client = client_create(None);
            let (service1, service1_key) = service_key_create(&client);
            let (_service2, service2_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service1_key.value));
            let user = user_create_with_password(
                &client,
                true,
                USER_NAME,
                &user_email,
                false,
                false,
                USER_PASSWORD,
            );
            let user_key = user_key_create(&client, KEY_NAME, KeyType::Key, service1.id, user);

            let client = client_create(Some(&service2_key.value));
            let body = AuthKeyRequest::new(&user_key.key, None);
            let res = client.auth_key_verify(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_auth_key_verify_bad_request_service_key() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let body = AuthKeyRequest::new(&service_key.value, None);
            let res = client.auth_key_verify(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_auth_key_verify_bad_request_unknown_user_key_key() {
            let client = client_create(None);
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user = user_create(&client, true, USER_NAME, &user_email);
            let user_key = user_key_create(&client, KEY_NAME, KeyType::Token, service.id, user);

            let body = AuthKeyRequest::new(&user_key.key, None);
            let res = client.auth_key_verify(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_auth_key_verify_ok() {
            let client = client_create(None);
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user = user_create(&client, true, USER_NAME, &user_email);
            let user_key = user_key_create(&client, KEY_NAME, KeyType::Key, service.id, user);

            let body = AuthKeyRequest::new(&user_key.key, None);
            client.auth_key_verify(body).unwrap();
        }

        #[test]
        #[ignore]
        fn api_auth_key_revoke_unauthorised() {
            let client = client_create(Some(INVALID_KEY));
            let body = AuthKeyRequest::new(INVALID_KEY, None);
            let res = client.auth_key_revoke(body).unwrap_err();
            assert_eq!(res, ClientError::Unauthorised);
        }

        #[test]
        #[ignore]
        fn api_auth_key_revoke_bad_request_invalid_key() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let body = AuthKeyRequest::new(INVALID_KEY, None);
            let res = client.auth_key_revoke(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_auth_key_revoke_bad_request_unknown_user_key_for_service() {
            let client = client_create(None);
            let (service1, service1_key) = service_key_create(&client);
            let (_service2, service2_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service1_key.value));
            let user = user_create_with_password(
                &client,
                true,
                USER_NAME,
                &user_email,
                false,
                false,
                USER_PASSWORD,
            );
            let user_key = user_key_create(&client, KEY_NAME, KeyType::Key, service1.id, user);

            let client = client_create(Some(&service2_key.value));
            let body = AuthKeyRequest::new(&user_key.key, None);
            let res = client.auth_key_revoke(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_auth_key_revoke_bad_request_service_key() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let body = AuthKeyRequest::new(&service_key.value, None);
            let res = client.auth_key_revoke(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_auth_key_revoke_bad_request_unknown_user_key_key() {
            let client = client_create(None);
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user = user_create(&client, true, USER_NAME, &user_email);
            let user_key = user_key_create(&client, KEY_NAME, KeyType::Token, service.id, user);

            let body = AuthKeyRequest::new(&user_key.key, None);
            let res = client.auth_key_revoke(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_auth_key_revoke_ok() {
            let client = client_create(None);
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user = user_create(&client, true, USER_NAME, &user_email);
            let user_key = user_key_create(&client, KEY_NAME, KeyType::Key, service.id, user);

            let body = AuthKeyRequest::new(&user_key.key, None);
            client.auth_key_revoke(body).unwrap();
            let body = AuthKeyRequest::new(&user_key.key, None);
            let res = client.auth_key_verify(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }
    };
}
