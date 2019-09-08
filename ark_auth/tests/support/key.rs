#[macro_export]
macro_rules! key_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_key_list_forbidden() {
            let client = client_create(Some(INVALID_KEY));
            let res = client
                .key_list(KeyListQuery {
                    gt: None,
                    lt: None,
                    limit: None,
                })
                .unwrap_err();
            assert_eq!(res, ClientError::Forbidden);
        }

        // TODO(test): Reimplement these tests.
        // #[test]
        // #[ignore]
        // fn api_key_list_bad_request_invalid_gt() {
        //     let client = client_create(None);
        //     let (_service, service_key) = service_key_create(&client);

        //     let client = client_create(Some(&service_key.value));
        //     let res = client
        //         .key_list(KeyListQuery {
        //             gt: Some("".to_owned()),
        //             lt: None,
        //             limit: None,
        //         })
        //         .unwrap_err();
        //     assert_eq!(res, ClientError::BadRequest);
        // }

        // #[test]
        // #[ignore]
        // fn api_key_list_bad_request_invalid_lt() {
        //     let client = client_create(None);
        //     let (_service, service_key) = service_key_create(&client);

        //     let client = client_create(Some(&service_key.value));
        //     let res = client
        //         .key_list(KeyListQuery {
        //             gt: None,
        //             lt: Some("".to_owned()),
        //             limit: None,
        //         })
        //         .unwrap_err();
        //     assert_eq!(res, ClientError::BadRequest);
        // }

        #[test]
        #[ignore]
        fn api_key_list_bad_request_invalid_limit() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);

            let client = client_create(Some(&service_key.value));
            let res = client
                .key_list(KeyListQuery {
                    gt: None,
                    lt: None,
                    limit: Some("-1".to_owned()),
                })
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
            let limit = "3";

            let body = KeyCreateBody::with_user_id(true, KEY_NAME, user.id);
            client.key_create(body.clone()).unwrap();
            client.key_create(body.clone()).unwrap();
            client.key_create(body.clone()).unwrap();
            client.key_create(body.clone()).unwrap();
            client.key_create(body.clone()).unwrap();

            let res1 = client
                .key_list(KeyListQuery {
                    gt: None,
                    lt: None,
                    limit: Some(limit.to_owned()),
                })
                .unwrap();
            assert_eq!(res1.data.len(), 3);
            let r1_1 = &res1.data[0];
            let r1_2 = &res1.data[1];
            let r1_3 = &res1.data[2];

            let res2 = client
                .key_list(KeyListQuery {
                    gt: Some(r1_1.to_owned()),
                    lt: None,
                    limit: Some(limit.to_owned()),
                })
                .unwrap();
            assert_eq!(res2.data.len(), 3);
            let r2_2 = &res2.data[0];
            let r2_3 = &res2.data[1];
            let r2_4 = &res2.data[2];
            assert_eq!(r2_2, r1_2);
            assert_eq!(r2_3, r1_3);

            let res3 = client
                .key_list(KeyListQuery {
                    gt: Some(r1_2.to_owned()),
                    lt: None,
                    limit: Some(limit.to_owned()),
                })
                .unwrap();
            assert_eq!(res3.data.len(), 3);
            let r3_3 = &res3.data[0];
            let r3_4 = &res3.data[1];
            let r3_5 = &res3.data[2];
            assert_eq!(r3_3, r2_3);
            assert_eq!(r3_4, r2_4);

            let res4 = client
                .key_list(KeyListQuery {
                    gt: None,
                    lt: Some(r3_5.to_owned()),
                    limit: Some(limit.to_owned()),
                })
                .unwrap();
            assert_eq!(res4.data.len(), 3);
            let r4_2 = &res4.data[0];
            let r4_3 = &res4.data[1];
            let r4_4 = &res4.data[2];
            assert_eq!(r4_2, r2_2);
            assert_eq!(r4_3, r3_3);
            assert_eq!(r4_4, r3_4);

            let res5 = client
                .key_list(KeyListQuery {
                    gt: None,
                    lt: Some(r4_4.to_owned()),
                    limit: Some(limit.to_owned()),
                })
                .unwrap();
            assert_eq!(res5.data.len(), 3);
            let r5_1 = &res5.data[0];
            let r5_2 = &res5.data[1];
            let r5_3 = &res5.data[2];
            assert_eq!(r5_1, r1_1);
            assert_eq!(r5_2, r4_2);
            assert_eq!(r5_3, r4_3);
        }

        #[test]
        #[ignore]
        fn api_key_create_forbidden() {
            let client = client_create(Some(INVALID_KEY));
            let body = KeyCreateBody::new(true, KEY_NAME);
            let res = client.key_create(body).unwrap_err();
            assert_eq!(res, ClientError::Forbidden);
        }

        #[test]
        #[ignore]
        fn api_key_read_forbidden() {
            let client = client_create(Some(INVALID_KEY));
            let res = client.key_read(Uuid::nil()).unwrap_err();
            assert_eq!(res, ClientError::Forbidden);
        }
    };
}
