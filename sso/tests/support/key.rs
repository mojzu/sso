#[macro_export]
macro_rules! key_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_key_list_unauthorised() {
            let client = client_create(Some(INVALID_KEY));
            let res = client
                .key_list(KeyListRequestBuilder::default().build().unwrap())
                .unwrap_err();
            assert_eq!(res, ClientError::Unauthorised);
        }

        #[test]
        #[ignore]
        fn api_key_list_bad_request_invalid_limit() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let res = client
                .key_list(
                    KeyListRequestBuilder::default()
                        .limit(Some(-1))
                        .build()
                        .unwrap(),
                )
                .unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_key_list_ok() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user = user_create(&client, true, USER_NAME, &user_email);
            let limit = 3;

            let body = KeyCreateRequest::with_user_id(true, KeyType::Key, KEY_NAME, user.id);
            client.key_create(body.clone()).unwrap();
            client.key_create(body.clone()).unwrap();
            client.key_create(body.clone()).unwrap();
            client.key_create(body.clone()).unwrap();
            client.key_create(body.clone()).unwrap();

            let res1 = client
                .key_list(
                    KeyListRequestBuilder::default()
                        .limit(Some(limit))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res1.data.len(), 3);
            let r1_1 = &res1.data[0].id;
            let r1_2 = &res1.data[1].id;
            let r1_3 = &res1.data[2].id;

            let res2 = client
                .key_list(
                    KeyListRequestBuilder::default()
                        .gt(Some(r1_1.to_owned()))
                        .limit(Some(limit))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res2.data.len(), 3);
            let r2_2 = &res2.data[0].id;
            let r2_3 = &res2.data[1].id;
            let r2_4 = &res2.data[2].id;
            assert_eq!(r2_2, r1_2);
            assert_eq!(r2_3, r1_3);

            let res3 = client
                .key_list(
                    KeyListRequestBuilder::default()
                        .gt(Some(r1_2.to_owned()))
                        .limit(Some(limit))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res3.data.len(), 3);
            let r3_3 = &res3.data[0].id;
            let r3_4 = &res3.data[1].id;
            let r3_5 = &res3.data[2].id;
            assert_eq!(r3_3, r2_3);
            assert_eq!(r3_4, r2_4);

            let res4 = client
                .key_list(
                    KeyListRequestBuilder::default()
                        .lt(Some(r3_5.to_owned()))
                        .limit(Some(limit))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res4.data.len(), 3);
            let r4_2 = &res4.data[0].id;
            let r4_3 = &res4.data[1].id;
            let r4_4 = &res4.data[2].id;
            assert_eq!(r4_2, r2_2);
            assert_eq!(r4_3, r3_3);
            assert_eq!(r4_4, r3_4);

            let res5 = client
                .key_list(
                    KeyListRequestBuilder::default()
                        .lt(Some(r4_4.to_owned()))
                        .limit(Some(limit))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res5.data.len(), 3);
            let r5_1 = &res5.data[0].id;
            let r5_2 = &res5.data[1].id;
            let r5_3 = &res5.data[2].id;
            assert_eq!(r5_1, r1_1);
            assert_eq!(r5_2, r4_2);
            assert_eq!(r5_3, r4_3);
        }

        #[test]
        #[ignore]
        fn api_key_create_unauthorised() {
            let client = client_create(Some(INVALID_KEY));
            let body = KeyCreateRequest::new(true, KeyType::Key, KEY_NAME);
            let res = client.key_create(body).unwrap_err();
            assert_eq!(res, ClientError::Unauthorised);
        }

        #[test]
        #[ignore]
        fn api_key_create_bad_request_multiple_active_token_keys() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user = user_create(&client, true, USER_NAME, &user_email);

            let body = KeyCreateRequest::with_user_id(true, KeyType::Token, KEY_NAME, user.id);
            client.key_create(body.clone()).unwrap();
            let res = client.key_create(body.clone()).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_key_create_bad_request_multiple_active_totp_keys() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user = user_create(&client, true, USER_NAME, &user_email);

            let body = KeyCreateRequest::with_user_id(true, KeyType::Totp, KEY_NAME, user.id);
            client.key_create(body.clone()).unwrap();
            let res = client.key_create(body.clone()).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_key_read_unauthorised() {
            let client = client_create(Some(INVALID_KEY));
            let res = client.key_read(Uuid::nil()).unwrap_err();
            assert_eq!(res, ClientError::Unauthorised);
        }

        #[test]
        #[ignore]
        fn api_key_read_ok() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let res = client.key_read(service_key.id).unwrap();
            assert_eq!(res.data.id, service_key.id);
        }
    };
}
