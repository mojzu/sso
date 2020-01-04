#[macro_export]
macro_rules! guide_integration_test {
    () => {
        #[test]
        #[ignore]
        fn guide_api_key() {
            let mut client = client_create(None);
            let (service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let user = user_create(&mut client, true, USER_NAME, &user_email);
            let (_, user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Key, service.id, user);

            user_key_verify(&mut client, &user_key);
            let body = pb::AuthKeyRequest::new(&user_key.value, None);
            client.auth_key_revoke(body).unwrap();
            user_key_verify_bad_request(&mut client, &user_key.value);
        }

        #[test]
        #[ignore]
        fn guide_login() {
            let mut client = client_create(None);
            let (service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let user = user_create_with_password(
                &mut client,
                true,
                USER_NAME,
                &user_email,
                false,
                false,
                USER_PASSWORD,
            );
            let (user, _) =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service.id, user);
            let user_token = auth_local_login(&mut client, &user.id, &user_email, USER_PASSWORD);

            user_token_verify(&mut client, &user_token);
            let user_token = user_token_refresh(&mut client, &user_token);
            let body = pb::AuthTokenRequest::new(&user_token.access.unwrap().token, None);
            client.auth_token_revoke(body).unwrap();

            let body = pb::AuthTokenRequest::new(&user_token.refresh.unwrap().token, None);
            let res = client.auth_token_verify(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
        }

        #[test]
        #[ignore]
        fn guide_reset_password() {
            let mut client = client_create(None);
            let (service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let user = user_create_with_password(
                &mut client,
                true,
                USER_NAME,
                &user_email,
                true,
                false,
                USER_PASSWORD,
            );
            let _user_key =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service.id, user);

            let body = pb::AuthResetPasswordRequest::new(&user_email);
            client.auth_local_reset_password(body).unwrap();
        }

        #[test]
        #[ignore]
        fn guide_oauth2_login() {
            let mut client = client_create(None);
            let (service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let user = user_create_with_password(
                &mut client,
                true,
                USER_NAME,
                &user_email,
                false,
                false,
                USER_PASSWORD,
            );
            let _user_key =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service.id, user);

            let res = client.auth_microsoft_oauth2_url(()).unwrap().into_inner();
            assert!(!res.url.is_empty());
        }
    };
}
