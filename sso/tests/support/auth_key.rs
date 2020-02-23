#[macro_export]
macro_rules! auth_key_integration_test {
    () => {
        #[test]
        #[ignore]
        fn auth_key_verify_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let body = pb::AuthKeyRequest::new(INVALID_KEY, None);
            let res = client.auth_key_verify(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_key_verify_bad_request_invalid_key() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthKeyRequest::new(INVALID_KEY, None);
            let res = client.auth_key_verify(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_key_verify_bad_request_unknown_user_key_for_service() {
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
            let (_user, user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Key, service1.id, user);

            let mut client = client_create(Some(&service2_key.value));
            let body = pb::AuthKeyRequest::new(&user_key.value, None);
            let res = client.auth_key_verify(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_key_verify_bad_request_service_key() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthKeyRequest::new(&service_key.value, None);
            let res = client.auth_key_verify(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_key_verify_bad_request_unknown_user_key_key() {
            let mut client = client_create(None);
            let (service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let user = user_create(&mut client, true, USER_NAME, &user_email);
            let (_user, user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service.id, user);

            let body = pb::AuthKeyRequest::new(&user_key.value, None);
            let res = client.auth_key_verify(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_key_verify_ok() {
            let mut client = client_create(None);
            let (service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let user = user_create(&mut client, true, USER_NAME, &user_email);
            let (_user, user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Key, service.id, user);

            let body = pb::AuthKeyRequest::new(&user_key.value, None);
            client.auth_key_verify(body).unwrap();
        }

        #[test]
        #[ignore]
        fn auth_key_revoke_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let body = pb::AuthKeyRequest::new(INVALID_KEY, None);
            let res = client.auth_key_revoke(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_key_revoke_bad_request_invalid_key() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthKeyRequest::new(INVALID_KEY, None);
            let res = client.auth_key_revoke(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_key_revoke_bad_request_unknown_user_key_for_service() {
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
            let (_user, user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Key, service1.id, user);

            let mut client = client_create(Some(&service2_key.value));
            let body = pb::AuthKeyRequest::new(&user_key.value, None);
            let res = client.auth_key_revoke(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_key_revoke_bad_request_service_key() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthKeyRequest::new(&service_key.value, None);
            let res = client.auth_key_revoke(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_key_revoke_bad_request_unknown_user_key_key() {
            let mut client = client_create(None);
            let (service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let user = user_create(&mut client, true, USER_NAME, &user_email);
            let (_user, user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service.id, user);

            let body = pb::AuthKeyRequest::new(&user_key.value, None);
            let res = client.auth_key_revoke(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_key_revoke_ok() {
            let mut client = client_create(None);
            let (service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let user = user_create(&mut client, true, USER_NAME, &user_email);
            let (_user, user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Key, service.id, user);

            let body = pb::AuthKeyRequest::new(&user_key.value, None);
            client.auth_key_revoke(body).unwrap();
            let body = pb::AuthKeyRequest::new(&user_key.value, None);
            let res = client.auth_key_verify(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }
    };
}
