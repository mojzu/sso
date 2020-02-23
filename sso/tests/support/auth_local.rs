#[macro_export]
macro_rules! auth_local_integration_test {
    () => {
        #[test]
        #[ignore]
        fn auth_local_login_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let user_email = email_create();

            let body = pb::AuthLoginRequest::new(&user_email, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_login_unauthorised_service_disabled() {
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
            let _user_key = user_key_create(
                &mut client,
                KEY_NAME,
                KeyType::Token,
                service.id.clone(),
                user,
            );

            let mut req = pb::ServiceUpdateRequest::default();
            req.id = service.id;
            req.is_enabled = Some(false);
            client.service_update(req).unwrap();

            let body = pb::AuthLoginRequest::new(&user_email, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_login_bad_request_invalid_email() {
            let mut client = client_create(None);

            let body = pb::AuthLoginRequest::new(INVALID_EMAIL, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_VALIDATION);
        }

        #[test]
        #[ignore]
        fn auth_local_login_bad_request_invalid_password() {
            let mut client = client_create(None);
            let user_email = email_create();

            let body = pb::AuthLoginRequest::new(&user_email, "");
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_VALIDATION);
        }

        #[test]
        #[ignore]
        fn auth_local_login_bad_request_unknown_email() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthLoginRequest::new(&user_email, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_login_bad_request_user_disabled() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let _user = user_create_with_password(
                &mut client,
                false,
                USER_NAME,
                &user_email,
                false,
                false,
                USER_PASSWORD,
            );

            let body = pb::AuthLoginRequest::new(&user_email, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_login_bad_request_unknown_user_key() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let _user = user_create_with_password(
                &mut client,
                true,
                USER_NAME,
                &user_email,
                false,
                false,
                USER_PASSWORD,
            );

            let body = pb::AuthLoginRequest::new(&user_email, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_login_bad_request_incorrect_password() {
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

            let body = pb::AuthLoginRequest::new(&user_email, USER_WRONG_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_login_bad_request_unknown_user_key_for_service() {
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
            let _user_key =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service1.id, user);

            let mut client = client_create(Some(&service2_key.value));
            let body = pb::AuthLoginRequest::new(&user_email, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_login_bad_request_unknown_user_token_key() {
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
            let _user_key = user_key_create(&mut client, KEY_NAME, KeyType::Key, service.id, user);

            let body = pb::AuthLoginRequest::new(&user_email, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_login_forbidden_user_password_require_update() {
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
                true,
                USER_PASSWORD,
            );
            let _user_key =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service.id, user);

            let body = pb::AuthLoginRequest::new(&user_email, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::PermissionDenied);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_login_ok() {
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

            let body = pb::AuthLoginRequest::new(&user_email, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap().into_inner();
            assert_eq!(res.user.unwrap().id, user.id);
        }

        #[test]
        #[ignore]
        fn auth_local_register_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let user_email = email_create();

            let body = pb::AuthRegisterRequest::new(USER_NAME, &user_email);
            let res = client.auth_local_register(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_register_unauthorised_service_disabled() {
            let mut client = client_create(None);
            let (service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let mut req = pb::ServiceUpdateRequest::default();
            req.id = service.id;
            req.is_enabled = Some(false);
            client.service_update(req).unwrap();

            let body = pb::AuthRegisterRequest::new(USER_NAME, &user_email);
            let res = client.auth_local_register(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_register_bad_request_invalid_name() {
            let mut client = client_create(None);
            let user_email = email_create();

            let body = pb::AuthRegisterRequest::new("", &user_email);
            let res = client.auth_local_register(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_VALIDATION);
        }

        #[test]
        #[ignore]
        fn auth_local_register_bad_request_invalid_email() {
            let mut client = client_create(None);

            let body = pb::AuthRegisterRequest::new(USER_NAME, INVALID_EMAIL);
            let res = client.auth_local_register(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_VALIDATION);
        }

        #[test]
        #[ignore]
        fn auth_local_register_bad_request_service_register_disabled() {
            let mut client = client_create(None);
            let (service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let mut req = pb::ServiceUpdateRequest::default();
            req.id = service.id;
            req.user_allow_register = Some(false);
            client.service_update(req).unwrap();

            let body = pb::AuthRegisterRequest::new(USER_NAME, &user_email);
            let res = client.auth_local_register(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_register_ok() {
            let mut client = client_create(None);
            let (service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let mut req = pb::ServiceUpdateRequest::default();
            req.id = service.id;
            req.user_allow_register = Some(true);
            client.service_update(req).unwrap();

            let body = pb::AuthRegisterRequest::new(USER_NAME, &user_email);
            client.auth_local_register(body).unwrap();
        }

        #[test]
        #[ignore]
        fn auth_local_register_user_exists_ok() {
            let mut client = client_create(None);
            let (service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let mut req = pb::ServiceUpdateRequest::default();
            req.id = service.id;
            req.user_allow_register = Some(true);
            client.service_update(req).unwrap();

            let body = pb::AuthRegisterRequest::new(USER_NAME, &user_email);
            client.auth_local_register(body).unwrap();

            let body = pb::AuthRegisterRequest::new(USER_NAME, &user_email);
            client.auth_local_register(body).unwrap();
        }

        #[test]
        #[ignore]
        fn auth_local_reset_password_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let user_email = email_create();

            let body = pb::AuthResetPasswordRequest::new(&user_email);
            let res = client.auth_local_reset_password(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_reset_password_bad_request_invalid_email() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthResetPasswordRequest::new(INVALID_EMAIL);
            let res = client.auth_local_reset_password(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_VALIDATION);
        }

        #[test]
        #[ignore]
        fn auth_local_reset_password_ok_unknown_email() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            // Endpoint should not infer users existence.
            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthResetPasswordRequest::new(&user_email);
            client.auth_local_reset_password(body).unwrap();
        }

        #[test]
        #[ignore]
        fn auth_local_reset_password_ok_unknown_user_key_for_service() {
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
                true,
                false,
                USER_PASSWORD,
            );
            let _user_key =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service1.id, user);

            // Endpoint should not infer users existence.
            let mut client = client_create(Some(&service2_key.value));
            let body = pb::AuthResetPasswordRequest::new(&user_email);
            client.auth_local_reset_password(body).unwrap();
        }

        #[test]
        #[ignore]
        fn auth_local_reset_password_ok_unknown_user_token_key() {
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
            let _user_key = user_key_create(&mut client, KEY_NAME, KeyType::Key, service.id, user);

            // Endpoint should not infer users existence.
            let body = pb::AuthResetPasswordRequest::new(&user_email);
            client.auth_local_reset_password(body).unwrap();
        }

        #[test]
        #[ignore]
        fn auth_local_reset_password_ok_reset_not_allowed() {
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

            // Endpoint should not infer users existence.
            let body = pb::AuthResetPasswordRequest::new(&user_email);
            client.auth_local_reset_password(body).unwrap();
        }

        #[test]
        #[ignore]
        fn auth_local_reset_password_ok() {
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
        fn auth_local_reset_password_confirm_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let body = pb::AuthResetPasswordConfirmRequest::new(INVALID_KEY, USER_PASSWORD);
            let res = client.auth_local_reset_password_confirm(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_reset_password_confirm_bad_request_invalid_token() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthResetPasswordConfirmRequest::new("", USER_PASSWORD);
            let res = client.auth_local_reset_password_confirm(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_VALIDATION);
        }

        #[test]
        #[ignore]
        fn auth_local_reset_password_confirm_bad_request_invalid_password() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthResetPasswordConfirmRequest::new(INVALID_KEY, "");
            let res = client.auth_local_reset_password_confirm(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_VALIDATION);
        }

        #[test]
        #[ignore]
        fn auth_local_update_email_forbidden_user_password_require_update() {
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
                true,
                USER_PASSWORD,
            );
            let (user, _user_key) =
                user_key_create(&mut client, KEY_NAME, KeyType::Token, service.id, user);

            let body = pb::AuthUpdateEmailRequest {
                email: user.email,
                password: USER_PASSWORD.to_owned(),
                new_email: email_create(),
            };
            let res = client.auth_local_update_email(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::PermissionDenied);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_update_password_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let body = pb::AuthUpdatePasswordRequest {
                email: String::from("test@test.com"),
                password: String::from(USER_PASSWORD),
                new_password: String::from(USER_PASSWORD),
            };
            let res = client.auth_local_update_password(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
            assert_eq!(res.message(), ERR_REDACTED);
        }

        #[test]
        #[ignore]
        fn auth_local_update_password_bad_request_invalid_email() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let body = pb::AuthUpdatePasswordRequest {
                email: String::from(INVALID_EMAIL),
                password: String::from(USER_PASSWORD),
                new_password: String::from(USER_PASSWORD),
            };
            let res = client.auth_local_update_password(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
            assert_eq!(res.message(), ERR_VALIDATION);
        }
    };
}
