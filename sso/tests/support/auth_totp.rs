#[macro_export]
macro_rules! auth_totp_integration_test {
    () => {
        #[test]
        #[ignore]
        fn auth_totp_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let body = pb::AuthTotpRequest::new(UUID_NIL, "123456");
            let res = client.auth_totp_verify(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_totp_bad_request_invalid_totp() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthTotpRequest::new(UUID_NIL, "");
            let res = client.auth_totp_verify(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_VALIDATION);
        }

        #[test]
        #[ignore]
        fn auth_totp_bad_request_unknown_user_totp_key() {
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
            let (user, user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Key, service.id, user);

            let totp = libreauth::oath::TOTPBuilder::new()
                .base32_key(&user_key.value)
                .finalize()
                .unwrap();
            let body = pb::AuthTotpRequest::new(user.id, totp.generate());
            let res = client.auth_totp_verify(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_totp_ok() {
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
            let (user, user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Totp, service.id, user);

            let totp = libreauth::oath::TOTPBuilder::new()
                .base32_key(&user_key.value)
                .finalize()
                .unwrap();
            let body = pb::AuthTotpRequest::new(user.id, totp.generate());
            client.auth_totp_verify(body).unwrap();
        }
    };
}
