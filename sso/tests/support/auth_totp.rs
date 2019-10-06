#[macro_export]
macro_rules! auth_totp_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_auth_totp_unauthorised() {
            let client = client_create(Some(INVALID_KEY));
            let body = AuthTotpRequest::new(Uuid::nil(), "123456");
            let res = client.auth_totp(body).unwrap_err();
            assert_eq!(res, ClientError::Unauthorised);
        }

        #[test]
        #[ignore]
        fn api_auth_totp_bad_request_invalid_totp() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let body = AuthTotpRequest::new(Uuid::nil(), "");
            let res = client.auth_totp(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_auth_totp_bad_request_unknown_user_totp_key() {
            let client = client_create(None);
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user = user_create_with_password(
                &client,
                true,
                USER_NAME,
                &user_email,
                false,
                false,
                USER_PASSWORD,
            );
            let user_key = user_key_create(&client, KEY_NAME, KeyType::Key, service.id, user.id);

            let totp = libreauth::oath::TOTPBuilder::new()
                .base32_key(&user_key.key)
                .finalize()
                .unwrap();
            let body = AuthTotpRequest::new(user.id, totp.generate());
            let res = client.auth_totp(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_auth_totp_ok() {
            let client = client_create(None);
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user = user_create_with_password(
                &client,
                true,
                USER_NAME,
                &user_email,
                false,
                false,
                USER_PASSWORD,
            );
            let user_key = user_key_create(&client, KEY_NAME, KeyType::Totp, service.id, user.id);

            let totp = libreauth::oath::TOTPBuilder::new()
                .base32_key(&user_key.key)
                .finalize()
                .unwrap();
            let body = AuthTotpRequest::new(user.id, totp.generate());
            client.auth_totp(body).unwrap();
        }
    };
}
