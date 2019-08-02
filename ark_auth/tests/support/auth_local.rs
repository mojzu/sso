#[macro_export]
macro_rules! auth_local_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_auth_local_login_forbidden() {
            let client = client_create(Some(INVALID_SERVICE_KEY));
            let user_email = email_create();

            let body = AuthLoginBody::new(&user_email, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::Forbidden));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_bad_request_invalid_email() {
            let client = client_create(None);

            let body = AuthLoginBody::new(INVALID_EMAIL, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_bad_request_invalid_password() {
            let client = client_create(None);
            let user_email = email_create();

            let body = AuthLoginBody::new(&user_email, "");
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_bad_request_unknown_email() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let body = AuthLoginBody::new(&user_email, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_bad_request_disabled_user() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let _user =
                user_create_with_password(&client, false, USER_NAME, &user_email, USER_PASSWORD);

            let body = AuthLoginBody::new(&user_email, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_bad_request_unknown_user_key() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let _user =
                user_create_with_password(&client, true, USER_NAME, &user_email, USER_PASSWORD);

            let body = AuthLoginBody::new(&user_email, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_bad_request_incorrect_password() {
            let client = client_create(None);
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user =
                user_create_with_password(&client, true, USER_NAME, &user_email, USER_PASSWORD);
            let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

            let body = AuthLoginBody::new(&user_email, INVALID_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_bad_request_unknown_user_key_for_service() {
            let client = client_create(None);
            let (service1, service1_key) = service_key_create(&client);
            let (_service2, service2_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service1_key.value));
            let user =
                user_create_with_password(&client, true, USER_NAME, &user_email, USER_PASSWORD);
            let _user_key = user_key_create(&client, KEY_NAME, &service1.id, &user.id);

            let client = client_create(Some(&service2_key.value));
            let body = AuthLoginBody::new(&user_email, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_ok() {
            let client = client_create(None);
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user =
                user_create_with_password(&client, true, USER_NAME, &user_email, USER_PASSWORD);
            let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

            let body = AuthLoginBody::new(&user_email, USER_PASSWORD);
            let res = client.auth_local_login(body).unwrap();
            assert_eq!(res.data.user_id, user.id);
        }

        #[test]
        #[ignore]
        fn api_auth_local_reset_password_forbidden() {
            let client = client_create(Some(INVALID_SERVICE_KEY));
            let user_email = email_create();

            let body = AuthResetPasswordBody::new(&user_email);
            let res = client.auth_local_reset_password(body).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::Forbidden));
        }

        #[test]
        #[ignore]
        fn api_auth_local_reset_password_bad_request_invalid_email() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let body = AuthResetPasswordBody::new(INVALID_EMAIL);
            let res = client.auth_local_reset_password(body).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_reset_password_ok_unknown_email() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            // Endpoint should not infer users existence.
            let client = client_create(Some(&service_key.value));
            let body = AuthResetPasswordBody::new(&user_email);
            client.auth_local_reset_password(body).unwrap();
        }

        #[test]
        #[ignore]
        fn api_auth_local_reset_password_ok_unknown_user_key_for_service() {
            let client = client_create(None);
            let (service1, service1_key) = service_key_create(&client);
            let (_service2, service2_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service1_key.value));
            let user =
                user_create_with_password(&client, true, USER_NAME, &user_email, USER_PASSWORD);
            let _user_key = user_key_create(&client, KEY_NAME, &service1.id, &user.id);

            // Endpoint should not infer users existence.
            let client = client_create(Some(&service2_key.value));
            let body = AuthResetPasswordBody::new(&user_email);
            client.auth_local_reset_password(body).unwrap();
        }

        #[test]
        #[ignore]
        fn api_auth_local_reset_password_ok() {
            let client = client_create(None);
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user =
                user_create_with_password(&client, true, USER_NAME, &user_email, USER_PASSWORD);
            let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

            let body = AuthResetPasswordBody::new(&user_email);
            client.auth_local_reset_password(body).unwrap();
        }

        #[test]
        #[ignore]
        fn api_auth_local_reset_password_confirm_forbidden() {
            let client = client_create(Some(INVALID_SERVICE_KEY));
            let body = AuthResetPasswordConfirmBody::new(INVALID_UUID, USER_PASSWORD);
            let res = client.auth_local_reset_password_confirm(body).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::Forbidden));
        }

        #[test]
        #[ignore]
        fn api_auth_local_reset_password_confirm_bad_request_invalid_token() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let body = AuthResetPasswordConfirmBody::new("", USER_PASSWORD);
            let res = client.auth_local_reset_password_confirm(body).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_reset_password_confirm_bad_request_invalid_password() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let body = AuthResetPasswordConfirmBody::new(INVALID_UUID, "");
            let res = client.auth_local_reset_password_confirm(body).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }
    };
}
