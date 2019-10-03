#[macro_export]
macro_rules! guide_integration_test {
    () => {
        #[test]
        #[ignore]
        fn guide_api_key() {
            let client = client_create(None);
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user = user_create(&client, true, USER_NAME, &user_email);
            let user_key = user_key_create(&client, KEY_NAME, service.id, user.id);

            user_key_verify(&client, &user_key);
            let body = AuthKeyRequest::new(&user_key.key, None);
            client.auth_key_revoke(body).unwrap();
            user_key_verify_bad_request(&client, &user_key.key);
        }

        #[test]
        #[ignore]
        fn guide_login() {
            let client = client_create(None);
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user =
                user_create_with_password(&client, true, USER_NAME, &user_email, USER_PASSWORD);
            let _user_key = user_key_create(&client, KEY_NAME, service.id, user.id);
            let user_token = auth_local_login(&client, user.id, &user_email, USER_PASSWORD);

            user_token_verify(&client, &user_token);
            let user_token = user_token_refresh(&client, &user_token);
            let body = AuthTokenRequest::new(&user_token.access_token, None);
            client.auth_token_revoke(body).unwrap();

            let body = AuthTokenRequest::new(&user_token.refresh_token, None);
            let res = client.auth_token_verify(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn guide_reset_password() {
            let client = client_create(None);
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user =
                user_create_with_password(&client, true, USER_NAME, &user_email, USER_PASSWORD);
            let _user_key = user_key_create(&client, KEY_NAME, service.id, user.id);

            let body = AuthResetPasswordRequest::new(&user_email);
            client.auth_local_reset_password(body).unwrap();
        }

        #[test]
        #[ignore]
        fn guide_oauth2_login() {
            let client = client_create(None);
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user =
                user_create_with_password(&client, true, USER_NAME, &user_email, USER_PASSWORD);
            let _user_key = user_key_create(&client, KEY_NAME, service.id, user.id);

            auth_microsoft_oauth2_url(&client);
        }
    };
}
