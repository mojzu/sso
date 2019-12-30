#[macro_export]
macro_rules! key_integration_test {
    () => {
        #[test]
        #[ignore]
        fn key_list_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let res = client.key_list(pb::KeyListRequest::default()).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
        }

        #[test]
        #[ignore]
        fn key_list_bad_request_invalid_limit() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let res = client.key_list(pb::KeyListRequest::limit(-1)).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
        }

        #[test]
        #[ignore]
        fn key_list_ok() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let user = user_create(&mut client, true, USER_NAME, &user_email);
            let limit = 3;

            let body = pb::KeyCreateRequest::with_user_id(true, KeyType::Key, KEY_NAME, user.id);
            let k1 = client
                .key_create(body.clone())
                .unwrap()
                .into_inner()
                .data
                .unwrap()
                .key
                .unwrap()
                .id;
            let k2 = client
                .key_create(body.clone())
                .unwrap()
                .into_inner()
                .data
                .unwrap()
                .key
                .unwrap()
                .id;
            let k3 = client
                .key_create(body.clone())
                .unwrap()
                .into_inner()
                .data
                .unwrap()
                .key
                .unwrap()
                .id;
            let k4 = client
                .key_create(body.clone())
                .unwrap()
                .into_inner()
                .data
                .unwrap()
                .key
                .unwrap()
                .id;
            let k5 = client
                .key_create(body.clone())
                .unwrap()
                .into_inner()
                .data
                .unwrap()
                .key
                .unwrap()
                .id;
            let id = vec![k1, k2, k3, k4, k5];

            let res1 = client
                .key_list(pb::KeyListRequest::limit_id(limit, id.clone()))
                .unwrap()
                .into_inner();
            assert_eq!(res1.data.len(), 3);
            let r1_1 = &res1.data[0].id;
            let r1_2 = &res1.data[1].id;
            let r1_3 = &res1.data[2].id;

            let res2 = client
                .key_list(pb::KeyListRequest::gt_limit_id(
                    r1_1.clone(),
                    limit,
                    id.clone(),
                ))
                .unwrap()
                .into_inner();
            assert_eq!(res2.data.len(), 3);
            let r2_2 = &res2.data[0].id;
            let r2_3 = &res2.data[1].id;
            let r2_4 = &res2.data[2].id;
            assert_eq!(r2_2, r1_2);
            assert_eq!(r2_3, r1_3);

            let res3 = client
                .key_list(pb::KeyListRequest::gt_limit_id(
                    r1_2.clone(),
                    limit,
                    id.clone(),
                ))
                .unwrap()
                .into_inner();
            assert_eq!(res3.data.len(), 3);
            let r3_3 = &res3.data[0].id;
            let r3_4 = &res3.data[1].id;
            let r3_5 = &res3.data[2].id;
            assert_eq!(r3_3, r2_3);
            assert_eq!(r3_4, r2_4);

            let res4 = client
                .key_list(pb::KeyListRequest::lt_limit_id(
                    r3_5.clone(),
                    limit,
                    id.clone(),
                ))
                .unwrap()
                .into_inner();
            assert_eq!(res4.data.len(), 3);
            let r4_2 = &res4.data[0].id;
            let r4_3 = &res4.data[1].id;
            let r4_4 = &res4.data[2].id;
            assert_eq!(r4_2, r2_2);
            assert_eq!(r4_3, r3_3);
            assert_eq!(r4_4, r3_4);

            let res5 = client
                .key_list(pb::KeyListRequest::lt_limit_id(
                    r4_4.clone(),
                    limit,
                    id.clone(),
                ))
                .unwrap()
                .into_inner();
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
        fn key_create_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let body = pb::KeyCreateRequest::new(true, KeyType::Key, KEY_NAME);
            let res = client.key_create(body).unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
        }

        #[test]
        #[ignore]
        fn key_create_bad_request_multiple_active_token_keys() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let user = user_create(&mut client, true, USER_NAME, &user_email);

            let body = pb::KeyCreateRequest::with_user_id(true, KeyType::Token, KEY_NAME, user.id);
            client.key_create(body.clone()).unwrap();
            let res = client.key_create(body.clone()).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
        }

        #[test]
        #[ignore]
        fn key_create_bad_request_multiple_active_totp_keys() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);
            let user_email = email_create();

            let mut client = client_create(Some(&service_key.value));
            let user = user_create(&mut client, true, USER_NAME, &user_email);

            let body = pb::KeyCreateRequest::with_user_id(true, KeyType::Totp, KEY_NAME, user.id);
            client.key_create(body.clone()).unwrap();
            let res = client.key_create(body.clone()).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
        }

        #[test]
        #[ignore]
        fn key_create_bad_request_invalid_service_id() {
            let mut client = client_create(None);
            let body = pb::KeyCreateRequest::with_service_id(
                true,
                KeyType::Key,
                KEY_NAME,
                UUID_NIL.to_owned(),
            );
            let res = client.key_create(body.clone()).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
        }

        #[test]
        #[ignore]
        fn key_create_bad_request_invalid_user_id() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key.value));
            let body = pb::KeyCreateRequest::with_user_id(
                true,
                KeyType::Key,
                KEY_NAME,
                UUID_NIL.to_owned(),
            );
            let res = client.key_create(body.clone()).unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
        }

        #[test]
        #[ignore]
        fn key_read_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let res = client
                .key_read(pb::KeyReadRequest {
                    id: UUID_NIL.to_owned(),
                    user_id: None,
                })
                .unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
        }

        #[test]
        #[ignore]
        fn key_read_not_found_service_mask() {
            let mut client = client_create(None);
            let (_service1, service1_key) = service_key_create(&mut client);
            let (_service2, service2_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service1_key.value));
            let res = client
                .key_read(pb::KeyReadRequest {
                    id: service2_key.key.unwrap().id,
                    user_id: None,
                })
                .unwrap_err();
            assert_eq!(res.code(), tonic::Code::NotFound);
        }

        #[test]
        #[ignore]
        fn key_read_ok() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);
            let service_key_id = service_key.key.unwrap().id;
            let key = client
                .key_read(pb::KeyReadRequest {
                    id: service_key_id.clone(),
                    user_id: None,
                })
                .unwrap()
                .into_inner()
                .data
                .unwrap();
            assert_eq!(key.id, service_key_id);
        }

        #[test]
        #[ignore]
        fn key_delete_not_found_service_mask() {
            let mut client = client_create(None);
            let (_service1, service1_key) = service_key_create(&mut client);
            let (_service2, service2_key) = service_key_create(&mut client);
            let service2_key_id = service2_key.key.unwrap().id;

            let mut client = client_create(Some(&service1_key.value));
            let res = client
                .key_delete(pb::KeyReadRequest {
                    id: service2_key_id,
                    user_id: None,
                })
                .unwrap_err();
            assert_eq!(res.code(), tonic::Code::NotFound);
        }
    };
}
