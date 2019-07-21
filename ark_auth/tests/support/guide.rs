#[macro_export]
macro_rules! guide_integration_test {
    () => {
        #[test]
        #[ignore]
        fn guide_api_key() {
            let mut client = client_create();
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service_key.value);
            let user = user_create(&client, true, USER_NAME, &user_email, None);
            let user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

            user_key_verify(&client, &user_key);
            client.auth_key_revoke(&user_key.key).unwrap();
            user_key_verify_bad_request(&client, &user_key.key);
        }

        #[test]
        #[ignore]
        fn guide_login() {
            let mut client = client_create();
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service_key.value);
            let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
            let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);
            let user_token = auth_local_login(&client, &user.id, &user_email, USER_PASSWORD);

            user_token_verify(&client, &user_token);
            let user_token = user_token_refresh(&client, &user_token);
            client.auth_token_revoke(&user_token.access_token).unwrap();

            let res = client
                .auth_token_verify(&user_token.refresh_token)
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn guide_reset_password() {
            let mut client = client_create();
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service_key.value);
            let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
            let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

            client.auth_local_reset_password(&user_email).unwrap();
        }

        #[test]
        #[ignore]
        fn guide_oauth2_login() {
            let mut client = client_create();
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service_key.value);
            let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
            let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

            auth_microsoft_oauth2_request(&client);
        }
    };
}
