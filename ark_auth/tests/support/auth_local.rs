#[macro_export]
macro_rules! auth_local_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_auth_local_login_forbidden() {
            let mut client = client_create();
            let user_email = email_create();

            client.options.set_authorisation(INVALID_SERVICE_KEY);
            let res = client
                .auth_local_login(&user_email, USER_PASSWORD)
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::Forbidden));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_bad_request_invalid_email() {
            let client = client_create();

            let res = client
                .auth_local_login(INVALID_EMAIL, USER_PASSWORD)
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_bad_request_invalid_password() {
            let client = client_create();
            let user_email = email_create();

            let res = client.auth_local_login(&user_email, "").unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_bad_request_unknown_email() {
            let mut client = client_create();
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service_key.value);
            let res = client
                .auth_local_login(&user_email, USER_PASSWORD)
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_bad_request_disabled_user() {
            let mut client = client_create();
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service_key.value);
            let _user = user_create(&client, false, USER_NAME, &user_email, Some(USER_PASSWORD));

            let res = client
                .auth_local_login(&user_email, USER_PASSWORD)
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_bad_request_unknown_user_key() {
            let mut client = client_create();
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service_key.value);
            let _user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));

            let res = client
                .auth_local_login(&user_email, USER_PASSWORD)
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_bad_request_incorrect_password() {
            let mut client = client_create();
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service_key.value);
            let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
            let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

            let res = client
                .auth_local_login(&user_email, INVALID_PASSWORD)
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_bad_request_unknown_user_key_for_service() {
            let mut client = client_create();
            let (service1, service1_key) = service_key_create(&client);
            let (_service2, service2_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service1_key.value);
            let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
            let _user_key = user_key_create(&client, KEY_NAME, &service1.id, &user.id);

            client.options.set_authorisation(&service2_key.value);
            let res = client
                .auth_local_login(&user_email, USER_PASSWORD)
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_login_ok() {
            let mut client = client_create();
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service_key.value);
            let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
            let _user_key = user_key_create(&client, KEY_NAME, &service.id, &user.id);

            let res = client.auth_local_login(&user_email, USER_PASSWORD).unwrap();
            assert_eq!(res.data.user_id, user.id);
        }

        #[test]
        #[ignore]
        fn api_auth_local_reset_password_forbidden() {
            let mut client = client_create();
            let user_email = email_create();

            client.options.set_authorisation(INVALID_SERVICE_KEY);
            let res = client.auth_local_reset_password(&user_email).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::Forbidden));
        }

        #[test]
        #[ignore]
        fn api_auth_local_reset_password_bad_request_invalid_email() {
            let mut client = client_create();
            let (_service, service_key) = service_key_create(&client);

            client.options.set_authorisation(&service_key.value);
            let res = client.auth_local_reset_password(INVALID_EMAIL).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_reset_password_ok_unknown_email() {
            let mut client = client_create();
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            // Endpoint should not infer users existence.
            client.options.set_authorisation(&service_key.value);
            client.auth_local_reset_password(&user_email).unwrap();
        }

        #[test]
        #[ignore]
        fn api_auth_local_reset_password_ok_unknown_user_key_for_service() {
            let mut client = client_create();
            let (service1, service1_key) = service_key_create(&client);
            let (_service2, service2_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service1_key.value);
            let user = user_create(&client, true, USER_NAME, &user_email, Some(USER_PASSWORD));
            let _user_key = user_key_create(&client, KEY_NAME, &service1.id, &user.id);

            // Endpoint should not infer users existence.
            client.options.set_authorisation(&service2_key.value);
            client.auth_local_reset_password(&user_email).unwrap();
        }

        #[test]
        #[ignore]
        fn api_auth_local_reset_password_ok() {
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
        fn api_auth_local_reset_password_confirm_forbidden() {
            let mut client = client_create();

            client.options.set_authorisation(INVALID_SERVICE_KEY);
            let res = client
                .auth_local_reset_password_confirm(INVALID_UUID, USER_PASSWORD)
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::Forbidden));
        }

        #[test]
        #[ignore]
        fn api_auth_local_reset_password_confirm_bad_request_invalid_token() {
            let mut client = client_create();
            let (_service, service_key) = service_key_create(&client);

            client.options.set_authorisation(&service_key.value);
            let res = client
                .auth_local_reset_password_confirm("", USER_PASSWORD)
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_auth_local_reset_password_confirm_bad_request_invalid_password() {
            let mut client = client_create();
            let (_service, service_key) = service_key_create(&client);

            client.options.set_authorisation(&service_key.value);
            let res = client
                .auth_local_reset_password_confirm(INVALID_UUID, "")
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }
    };
}
