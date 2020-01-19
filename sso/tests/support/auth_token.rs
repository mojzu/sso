#[macro_export]
macro_rules! auth_token_integration_test {
    () => {
        #[test]
        #[ignore]
        fn auth_token_verify_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let body = pb::AuthTokenRequest::new(INVALID_KEY, None);
            let res = client.auth_token_verify(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
            assert_eq!(res.message(), util::ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_token_verify_bad_request_invalid_token() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthTokenRequest::new(INVALID_KEY, None);
            let res = client.auth_token_verify(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), util::ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_token_verify_bad_request_unknown_user_key_for_service() {
            let mut client = client_create(None);
            let (service1, service1_key) = service_key_create(&mut client);
            let (_service2, service2_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service1_key.value));
            let user = user_create_with_password(
                &mut client,
                true,
                USER_NAME,
                &user_email,
                false,
                false,
                USER_PASSWORD,
            );
            let (user, _user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service1.id, user);
            let user_token = auth_local_login(&mut client, &user.id, &user_email, USER_PASSWORD);

            let mut client = client_create(Some(&service2_key.value));
            let body = pb::AuthTokenRequest::new(&user_token.access.unwrap().token, None);
            let res = client.auth_token_verify(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), util::ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_token_verify_ok() {
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
            let (user, _user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service.id, user);
            let user_token = auth_local_login(&mut client, &user.id, &user_email, USER_PASSWORD);

            let body = pb::AuthTokenRequest::new(&user_token.access.unwrap().token, None);
            client.auth_token_verify(body).unwrap();
        }

        #[test]
        #[ignore]
        fn auth_token_refresh_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let body = pb::AuthTokenRequest::new(INVALID_KEY, None);
            let res = client.auth_token_refresh(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
            assert_eq!(res.message(), util::ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_token_refresh_bad_request_invalid_token() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthTokenRequest::new(INVALID_KEY, None);
            let res = client.auth_token_refresh(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), util::ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_token_refresh_bad_request_unknown_user_key_for_service() {
            let mut client = client_create(None);
            let (service1, service1_key) = service_key_create(&mut client);
            let (_service2, service2_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service1_key.value));
            let user = user_create_with_password(
                &mut client,
                true,
                USER_NAME,
                &user_email,
                false,
                false,
                USER_PASSWORD,
            );
            let (user, _user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service1.id, user);
            let user_token = auth_local_login(&mut client, &user.id, &user_email, USER_PASSWORD);

            let mut client = client_create(Some(&service2_key.value));
            let body = pb::AuthTokenRequest::new(&user_token.refresh.unwrap().token, None);
            let res = client.auth_token_refresh(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), util::ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_token_refresh_bad_request_used_refresh_token() {
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
            let (user, _user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service.id, user);
            let user_token = auth_local_login(&mut client, &user.id, &user_email, USER_PASSWORD);

            user_token_verify(&mut client, &user_token);
            let user_token2 = user_token_refresh(&mut client, &user_token);
            let body = pb::AuthTokenRequest::new(&user_token2.access.unwrap().token, None);
            client.auth_token_verify(body).unwrap();

            let body = pb::AuthTokenRequest::new(&user_token.refresh.unwrap().token, None);
            let res = client.auth_token_refresh(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), util::ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_token_refresh_ok() {
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
            let (user, _user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service.id, user);
            let user_token = auth_local_login(&mut client, &user.id, &user_email, USER_PASSWORD);

            user_token_verify(&mut client, &user_token);
            let user_token = user_token_refresh(&mut client, &user_token);
            let body = pb::AuthTokenRequest::new(&user_token.access.unwrap().token, None);
            client.auth_token_verify(body).unwrap();
        }

        #[test]
        #[ignore]
        fn auth_token_revoke_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let body = pb::AuthTokenRequest::new(INVALID_KEY, None);
            let res = client.auth_token_revoke(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
            assert_eq!(res.message(), util::ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_token_revoke_bad_request_invalid_token() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthTokenRequest::new(INVALID_KEY, None);
            let res = client.auth_token_revoke(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), util::ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_token_revoke_bad_request_unknown_user_key_for_service() {
            let mut client = client_create(None);
            let (service1, service1_key) = service_key_create(&mut client);
            let (_service2, service2_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service1_key.value));
            let user = user_create_with_password(
                &mut client,
                true,
                USER_NAME,
                &user_email,
                false,
                false,
                USER_PASSWORD,
            );
            let (user, _user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service1.id, user);
            let user_token = auth_local_login(&mut client, &user.id, &user_email, USER_PASSWORD);

            let mut client = client_create(Some(&service2_key.value));
            let body = pb::AuthTokenRequest::new(&user_token.refresh.unwrap().token, None);
            let res = client.auth_token_revoke(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), util::ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_token_revoke_ok() {
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
            let (user, _user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service.id, user);

            let user_token = auth_local_login(&mut client, &user.id, &user_email, USER_PASSWORD);

            user_token_verify(&mut client, &user_token);
            let token_access = user_token.access.unwrap();
            let body = pb::AuthTokenRequest::new(&token_access.token, None);
            client.auth_token_revoke(body).unwrap();
            let body = pb::AuthTokenRequest::new(&token_access.token, None);
            let res = client.auth_token_verify(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), util::ERR_REDACTED);
        }
    };
}
