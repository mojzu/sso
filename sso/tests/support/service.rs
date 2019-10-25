#[macro_export]
macro_rules! service_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_service_list_ok() {
            let client = client_create(None);
            let (s1, _) = service_key_create(&client);
            let (s2, _) = service_key_create(&client);
            let (s3, _) = service_key_create(&client);
            let (s4, _) = service_key_create(&client);
            let (s5, _) = service_key_create(&client);
            let id = vec![s1.id, s2.id, s3.id, s4.id, s5.id];
            let limit = 3;

            let res1 = client
                .service_list(
                    ServiceListRequestBuilder::default()
                        .limit(Some(limit))
                        .id(Some(id.clone()))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res1.data.len(), 3);
            let r1_1 = &res1.data[0].id;
            let r1_2 = &res1.data[1].id;
            let r1_3 = &res1.data[2].id;

            let res2 = client
                .service_list(
                    ServiceListRequestBuilder::default()
                        .gt(Some(r1_1.to_owned()))
                        .limit(Some(limit))
                        .id(Some(id.clone()))
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
                .service_list(
                    ServiceListRequestBuilder::default()
                        .gt(Some(r1_2.to_owned()))
                        .limit(Some(limit))
                        .id(Some(id.clone()))
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
                .service_list(
                    ServiceListRequestBuilder::default()
                        .lt(Some(r3_5.to_owned()))
                        .limit(Some(limit))
                        .id(Some(id.clone()))
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
                .service_list(
                    ServiceListRequestBuilder::default()
                        .lt(Some(r4_4.to_owned()))
                        .limit(Some(limit))
                        .id(Some(id.clone()))
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
        fn api_service_read_unauthorised() {
            let client = client_create(Some(INVALID_KEY));
            let res = client.service_read(Uuid::nil()).unwrap_err();
            assert_eq!(res, ClientError::Unauthorised);
        }

        #[test]
        #[ignore]
        fn api_service_read_not_found_service_mask() {
            let client = client_create(None);
            let (_service1, service1_key) = service_key_create(&client);
            let (service2, _service2_key) = service_key_create(&client);

            let client = client_create(Some(&service1_key.value));
            let res = client.service_read(service2.id).unwrap_err();
            assert_eq!(res, ClientError::NotFound);
        }

        #[test]
        #[ignore]
        fn api_service_read_ok() {
            let client = client_create(None);
            let (service, _service_key) = service_key_create(&client);
            let res = client.service_read(service.id).unwrap();
            assert_eq!(res.data.id, service.id);
        }

        #[test]
        #[ignore]
        fn api_service_delete_not_found_service_mask() {
            let client = client_create(None);
            let (_service1, service1_key) = service_key_create(&client);
            let (service2, _service2_key) = service_key_create(&client);

            let client = client_create(Some(&service1_key.value));
            let res = client.service_delete(service2.id).unwrap_err();
            assert_eq!(res, ClientError::NotFound);
        }
    };
}
