#[macro_export]
macro_rules! user_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_user_list_forbidden() {
            let client = client_create(Some(INVALID_SERVICE_KEY));
            let res = client
                .user_list(UserListQuery {
                    gt: None,
                    lt: None,
                    limit: None,
                    email_eq: None,
                })
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::Forbidden));
        }

        #[test]
        #[ignore]
        fn api_user_list_ok() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user1_email = email_create();
            let user2_email = email_create();
            let user3_email = email_create();
            let user4_email = email_create();
            let user5_email = email_create();
            let limit = "3";

            let client = client_create(Some(&service_key.value));
            user_create(&client, true, USER_NAME, &user1_email, None);
            user_create(&client, true, USER_NAME, &user2_email, None);
            user_create(&client, true, USER_NAME, &user3_email, None);
            user_create(&client, true, USER_NAME, &user4_email, None);
            user_create(&client, true, USER_NAME, &user5_email, None);

            let res1 = client
                .user_list(UserListQuery {
                    gt: None,
                    lt: None,
                    limit: Some(limit.to_owned()),
                    email_eq: None,
                })
                .unwrap();
            assert_eq!(res1.data.len(), 3);
            let r1_1 = &res1.data[0];
            let r1_2 = &res1.data[1];
            let r1_3 = &res1.data[2];

            let res2 = client
                .user_list(UserListQuery {
                    gt: Some(r1_1.to_owned()),
                    lt: None,
                    limit: Some(limit.to_owned()),
                    email_eq: None,
                })
                .unwrap();
            assert_eq!(res2.data.len(), 3);
            let r2_2 = &res2.data[0];
            let r2_3 = &res2.data[1];
            let r2_4 = &res2.data[2];
            assert_eq!(r2_2, r1_2);
            assert_eq!(r2_3, r1_3);

            let res3 = client
                .user_list(UserListQuery {
                    gt: Some(r1_2.to_owned()),
                    lt: None,
                    limit: Some(limit.to_owned()),
                    email_eq: None,
                })
                .unwrap();
            assert_eq!(res3.data.len(), 3);
            let r3_3 = &res3.data[0];
            let r3_4 = &res3.data[1];
            let r3_5 = &res3.data[2];
            assert_eq!(r3_3, r2_3);
            assert_eq!(r3_4, r2_4);

            let res4 = client
                .user_list(UserListQuery {
                    gt: None,
                    lt: Some(r3_5.to_owned()),
                    limit: Some(limit.to_owned()),
                    email_eq: None,
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
                .user_list(UserListQuery {
                    gt: None,
                    lt: Some(r4_4.to_owned()),
                    limit: Some(limit.to_owned()),
                    email_eq: None,
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
        fn api_user_list_email_eq_ok() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user = user_create(&client, true, USER_NAME, &user_email, None);

            let res = client
                .user_list(UserListQuery {
                    gt: None,
                    lt: None,
                    limit: None,
                    email_eq: Some(user.email),
                })
                .unwrap();
            assert_eq!(res.data.len(), 1);
            assert_eq!(res.data[0], user.id);
        }

        #[test]
        #[ignore]
        fn api_user_create_ok() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            user_create(&client, true, USER_NAME, &user_email, None);
        }

        #[test]
        #[ignore]
        fn api_user_create_forbidden() {
            let client = client_create(Some(INVALID_SERVICE_KEY));
            let user_email = email_create();
            let res = client
                .user_create(true, USER_NAME, &user_email, None)
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::Forbidden));
        }

        #[test]
        #[ignore]
        fn api_user_create_bad_request_duplicate_user_email() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            user_create(&client, true, USER_NAME, &user_email, None);

            let res = client
                .user_create(true, USER_NAME, &user_email, None)
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }
    };
}
