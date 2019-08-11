#[macro_export]
macro_rules! auth_key_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_auth_key_verify_forbidden() {
            let client = client_create(Some(INVALID_SERVICE_KEY));
            let body = AuthKeyBody::new(INVALID_UUID, None);
            let res = client.auth_key_verify(body).unwrap_err();
            assert_eq!(res, ClientError::Forbidden);
        }

        #[test]
        #[ignore]
        fn api_auth_key_verify_bad_request_invalid_key() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let body = AuthKeyBody::new(INVALID_UUID, None);
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
            let user =
                user_create_with_password(&client, true, USER_NAME, &user_email, USER_PASSWORD);
            let user_key = user_key_create(&client, KEY_NAME, &service1.id, &user.id);

            let client = client_create(Some(&service2_key.value));
            let body = AuthKeyBody::new(&user_key.key, None);
            let res = client.auth_key_verify(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_auth_key_verify_bad_request_service_key() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let body = AuthKeyBody::new(&service_key.value, None);
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
            let user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

            let body = AuthKeyBody::new(&user_key.key, None);
            client.auth_key_verify(body).unwrap();
        }

        #[test]
        #[ignore]
        fn api_auth_key_revoke_forbidden() {
            let client = client_create(Some(INVALID_SERVICE_KEY));
            let body = AuthKeyBody::new(INVALID_UUID, None);
            let res = client.auth_key_revoke(body).unwrap_err();
            assert_eq!(res, ClientError::Forbidden);
        }

        #[test]
        #[ignore]
        fn api_auth_key_revoke_bad_request_invalid_key() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let body = AuthKeyBody::new(INVALID_UUID, None);
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
            let user =
                user_create_with_password(&client, true, USER_NAME, &user_email, USER_PASSWORD);
            let user_key = user_key_create(&client, KEY_NAME, &service1.id, &user.id);

            let client = client_create(Some(&service2_key.value));
            let body = AuthKeyBody::new(&user_key.key, None);
            let res = client.auth_key_revoke(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_auth_key_revoke_bad_request_service_key() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let body = AuthKeyBody::new(&service_key.value, None);
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
            let user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

            let body = AuthKeyBody::new(&user_key.key, None);
            client.auth_key_revoke(body).unwrap();
            let body = AuthKeyBody::new(&user_key.key, None);
            let res = client.auth_key_verify(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }
    };
}
