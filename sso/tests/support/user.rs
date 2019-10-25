#[macro_export]
macro_rules! user_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_user_list_unauthorised() {
            let client = client_create(Some(INVALID_KEY));
            let res = client
                .user_list(UserListRequestBuilder::default().build().unwrap())
                .unwrap_err();
            assert_eq!(res, ClientError::Unauthorised);
        }

        #[test]
        #[ignore]
        fn api_user_list_id_ok() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user1_email = email_create();
            let user2_email = email_create();
            let user3_email = email_create();
            let user4_email = email_create();
            let user5_email = email_create();
            let limit = 3;

            let client = client_create(Some(&service_key.value));
            user_create(&client, true, USER_NAME, &user1_email);
            user_create(&client, true, USER_NAME, &user2_email);
            user_create(&client, true, USER_NAME, &user3_email);
            user_create(&client, true, USER_NAME, &user4_email);
            user_create(&client, true, USER_NAME, &user5_email);

            let res1 = client
                .user_list(
                    UserListRequestBuilder::default()
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
                .user_list(
                    UserListRequestBuilder::default()
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
                .user_list(
                    UserListRequestBuilder::default()
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
                .user_list(
                    UserListRequestBuilder::default()
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
                .user_list(
                    UserListRequestBuilder::default()
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
        fn api_user_list_name_ok() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user1_email = email_create();
            let user2_email = email_create();
            let user3_email = email_create();
            let user4_email = email_create();
            let user5_email = email_create();
            let limit = 3;

            let client = client_create(Some(&service_key.value));
            let u1 = user_create(&client, true, "eee", &user1_email).id;
            let u2 = user_create(&client, true, "ddd", &user2_email).id;
            let u3 = user_create(&client, true, "ccc", &user3_email).id;
            let u4 = user_create(&client, true, "bbb", &user4_email).id;
            let u5 = user_create(&client, true, "aaa", &user5_email).id;
            let id = vec![u1, u2, u3, u4, u5];

            let res1 = client
                .user_list(
                    UserListRequestBuilder::default()
                        .name_ge(Some("_".to_owned()))
                        .limit(Some(limit))
                        .id(Some(id.clone()))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res1.data.len(), 3);
            let r1_1 = &res1.data[0];
            let r1_2 = &res1.data[1];
            let r1_3 = &res1.data[2];
            assert_eq!(r1_1.name, "aaa");
            assert_eq!(r1_2.name, "bbb");
            assert_eq!(r1_3.name, "ccc");

            let res2 = client
                .user_list(
                    UserListRequestBuilder::default()
                        .name_ge(Some(r1_1.name.to_owned()))
                        .offset_id(Some(r1_1.id))
                        .limit(Some(limit))
                        .id(Some(id.clone()))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res2.data.len(), 3);
            let r2_2 = &res2.data[0];
            let r2_3 = &res2.data[1];
            let r2_4 = &res2.data[2];
            assert_eq!(r2_2.id, r1_2.id);
            assert_eq!(r2_3.id, r1_3.id);
            assert_eq!(r2_2.name, "bbb");
            assert_eq!(r2_3.name, "ccc");
            assert_eq!(r2_4.name, "ddd");

            let res3 = client
                .user_list(
                    UserListRequestBuilder::default()
                        .name_ge(Some(r1_2.name.to_owned()))
                        .offset_id(Some(r1_2.id))
                        .limit(Some(limit))
                        .id(Some(id.clone()))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res3.data.len(), 3);
            let r3_3 = &res3.data[0];
            let r3_4 = &res3.data[1];
            let r3_5 = &res3.data[2];
            assert_eq!(r3_3.id, r2_3.id);
            assert_eq!(r3_4.id, r2_4.id);
            assert_eq!(r3_3.name, "ccc");
            assert_eq!(r3_4.name, "ddd");
            assert_eq!(r3_5.name, "eee");

            let res4 = client
                .user_list(
                    UserListRequestBuilder::default()
                        .name_le(Some(r3_5.name.to_owned()))
                        .offset_id(Some(r3_5.id))
                        .limit(Some(limit))
                        .id(Some(id.clone()))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res4.data.len(), 3);
            let r4_2 = &res4.data[0];
            let r4_3 = &res4.data[1];
            let r4_4 = &res4.data[2];
            assert_eq!(r4_2.id, r2_2.id);
            assert_eq!(r4_3.id, r3_3.id);
            assert_eq!(r4_4.id, r3_4.id);
            assert_eq!(r4_2.name, "bbb");
            assert_eq!(r4_3.name, "ccc");
            assert_eq!(r4_4.name, "ddd");

            let res5 = client
                .user_list(
                    UserListRequestBuilder::default()
                        .name_le(Some(r4_4.name.to_owned()))
                        .offset_id(Some(r4_4.id))
                        .limit(Some(limit))
                        .id(Some(id.clone()))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res5.data.len(), 3);
            let r5_1 = &res5.data[0];
            let r5_2 = &res5.data[1];
            let r5_3 = &res5.data[2];
            assert_eq!(r5_1.id, r1_1.id);
            assert_eq!(r5_2.id, r4_2.id);
            assert_eq!(r5_3.id, r4_3.id);
            assert_eq!(r5_1.name, "aaa");
            assert_eq!(r5_2.name, "bbb");
            assert_eq!(r5_3.name, "ccc");
        }

        #[test]
        #[ignore]
        fn api_user_list_filter_id_ok() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user = user_create(&client, true, USER_NAME, &user_email);

            let res = client
                .user_list(
                    UserListRequestBuilder::default()
                        .id(Some(vec![user.id]))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res.data.len(), 1);
            assert_eq!(res.data[0].id, user.id);
        }

        #[test]
        #[ignore]
        fn api_user_list_filter_email_ok() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user = user_create(&client, true, USER_NAME, &user_email);

            let res = client
                .user_list(
                    UserListRequestBuilder::default()
                        .email(Some(vec![user.email]))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res.data.len(), 1);
            assert_eq!(res.data[0].id, user.id);
        }

        #[test]
        #[ignore]
        fn api_user_create_ok() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            user_create(&client, true, USER_NAME, &user_email);
        }

        #[test]
        #[ignore]
        fn api_user_create_unauthorised() {
            let client = client_create(Some(INVALID_KEY));
            let user_email = email_create();
            let body = UserCreateRequest::new(true, USER_NAME, &user_email);
            let res = client.user_create(body).unwrap_err();
            assert_eq!(res, ClientError::Unauthorised);
        }

        #[test]
        #[ignore]
        fn api_user_create_bad_request_duplicate_user_email() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            user_create(&client, true, USER_NAME, &user_email);

            let body = UserCreateRequest::new(true, USER_NAME, &user_email);
            let res = client.user_create(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_user_create_bad_request_invalid_locale() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let body = UserCreateRequest::new(true, USER_NAME, &user_email).locale("");
            let res = client.user_create(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_user_create_bad_request_invalid_timezone() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let body = UserCreateRequest::new(true, USER_NAME, &user_email).timezone("invalid");
            let res = client.user_create(body).unwrap_err();
            assert_eq!(res, ClientError::BadRequest);
        }

        #[test]
        #[ignore]
        fn api_user_read_unauthorised() {
            let client = client_create(Some(INVALID_KEY));
            let res = client.user_read(Uuid::nil()).unwrap_err();
            assert_eq!(res, ClientError::Unauthorised);
        }

        #[test]
        #[ignore]
        fn api_user_read_ok() {
            let client = client_create(None);
            let user_email = email_create();
            let user = user_create(&client, true, USER_NAME, &user_email);
            let res = client.user_read(user.id).unwrap();
            assert_eq!(res.data.id, user.id);
        }

        #[test]
        #[ignore]
        fn api_user_update_unauthorised() {
            let client = client_create(Some(INVALID_KEY));
            let res = client
                .user_update(Uuid::nil(), UserUpdateRequest::default())
                .unwrap_err();
            assert_eq!(res, ClientError::Unauthorised);
        }

        #[test]
        #[ignore]
        fn api_user_update_ok() {
            let client = client_create(None);
            let (service, service_key) = service_key_create(&client);
            let user_email = email_create();

            let client = client_create(Some(&service_key.value));
            let user1 = user_create(&client, true, USER_NAME, &user_email);

            let user2 = client
                .user_update(user1.id, UserUpdateRequest::default().is_enabled(false))
                .unwrap()
                .data;
            assert_eq!(user1.id, user2.id);
            assert_ne!(user1.updated_at, user2.updated_at);

            let update_type = AuditType::UserUpdate.to_string().unwrap();
            let audit_list = client
                .audit_list(
                    AuditListRequestBuilder::default()
                        .type_(Some(vec![update_type]))
                        .subject(Some(vec![user2.id.to_string()]))
                        .build()
                        .unwrap(),
                )
                .unwrap()
                .data;
            assert_eq!(audit_list.len(), 1);
            let audit = &audit_list[0];
            println!("{:?}", audit); // TODO(refactor): Clean this up.
            assert_eq!(audit.service_id.unwrap(), service.id);
        }
    };
}
