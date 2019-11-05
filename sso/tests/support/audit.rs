#[macro_export]
macro_rules! audit_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_audit_list_created_ok() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let client = client_create(Some(&service_key.value));
            let type_ = "test_1".to_owned();
            let read = AuditReadRequest::new(None);
            let limit = 3;

            let a1 = client
                .audit_create(
                    AuditCreateRequestBuilder::default()
                        .type_(type_.clone())
                        .build()
                        .unwrap(),
                )
                .unwrap()
                .data;
            client
                .audit_create(
                    AuditCreateRequestBuilder::default()
                        .type_(type_.clone())
                        .build()
                        .unwrap(),
                )
                .unwrap();
            client
                .audit_create(
                    AuditCreateRequestBuilder::default()
                        .type_(type_.clone())
                        .build()
                        .unwrap(),
                )
                .unwrap();
            client
                .audit_create(
                    AuditCreateRequestBuilder::default()
                        .type_(type_.clone())
                        .build()
                        .unwrap(),
                )
                .unwrap();
            client
                .audit_create(
                    AuditCreateRequestBuilder::default()
                        .type_(type_.clone())
                        .build()
                        .unwrap(),
                )
                .unwrap();

            let res1 = client
                .audit_list(
                    AuditListRequestBuilder::default()
                        .ge(Some(a1.created_at))
                        .limit(Some(limit))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res1.data.len(), 3);
            let r1_1 = res1.data[0].id;
            let r1_2 = res1.data[1].id;
            let r1_3 = res1.data[2].id;
            assert_eq!(r1_1, a1.id);
            let a1 = client.audit_read(r1_1, read.clone()).unwrap().data;

            let res2 = client
                .audit_list(
                    AuditListRequestBuilder::default()
                        .ge(Some(a1.created_at))
                        .offset_id(Some(a1.id))
                        .limit(Some(limit))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res2.data.len(), 3);
            let r2_2 = res2.data[0].id;
            let r2_3 = res2.data[1].id;
            let r2_4 = res2.data[2].id;
            assert_eq!(r2_2, r1_2);
            assert_eq!(r2_3, r1_3);
            let a2 = client.audit_read(r2_2, read.clone()).unwrap().data;

            let res3 = client
                .audit_list(
                    AuditListRequestBuilder::default()
                        .ge(Some(a2.created_at))
                        .offset_id(Some(a2.id))
                        .limit(Some(limit))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res3.data.len(), 3);
            let r3_3 = res3.data[0].id;
            let r3_4 = res3.data[1].id;
            let r3_5 = res3.data[2].id;
            assert_eq!(r3_3, r2_3);
            assert_eq!(r3_4, r2_4);
            let a5 = client.audit_read(r3_5, read.clone()).unwrap().data;

            let res4 = client
                .audit_list(
                    AuditListRequestBuilder::default()
                        .le(Some(a5.created_at))
                        .offset_id(Some(a5.id))
                        .limit(Some(limit))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res4.data.len(), 3);
            let r4_2 = res4.data[0].id;
            let r4_3 = res4.data[1].id;
            let r4_4 = res4.data[2].id;
            assert_eq!(r4_2, r2_2);
            assert_eq!(r4_3, r3_3);
            assert_eq!(r4_4, r3_4);
            let a4 = client.audit_read(r4_4, read.clone()).unwrap().data;

            let res5 = client
                .audit_list(
                    AuditListRequestBuilder::default()
                        .le(Some(a4.created_at))
                        .offset_id(Some(a4.id))
                        .limit(Some(limit))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res5.data.len(), 3);
            let r5_1 = res5.data[0].id;
            let r5_2 = res5.data[1].id;
            let r5_3 = res5.data[2].id;
            assert_eq!(r5_1, r1_1);
            assert_eq!(r5_2, r4_2);
            assert_eq!(r5_3, r4_3);
        }

        #[test]
        #[ignore]
        fn api_audit_list_created_and_ok() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let client = client_create(Some(&service_key.value));
            let type_ = "test_1".to_owned();
            let read = AuditReadRequest::new(None);
            let limit = 3;

            let a1 = client
                .audit_create(
                    AuditCreateRequestBuilder::default()
                        .type_(type_.to_owned())
                        .build()
                        .unwrap(),
                )
                .unwrap()
                .data;
            client
                .audit_create(
                    AuditCreateRequestBuilder::default()
                        .type_(type_.to_owned())
                        .build()
                        .unwrap(),
                )
                .unwrap();
            client
                .audit_create(
                    AuditCreateRequestBuilder::default()
                        .type_(type_.to_owned())
                        .build()
                        .unwrap(),
                )
                .unwrap();

            let res1 = client
                .audit_list(
                    AuditListRequestBuilder::default()
                        .ge(Some(a1.created_at))
                        .limit(Some(limit))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res1.data.len(), 3);
            let r1_1 = res1.data[0].id;
            let r1_2 = res1.data[1].id;
            let r1_3 = res1.data[2].id;
            assert_eq!(r1_1, a1.id);
            let a1 = client.audit_read(r1_1, read.clone()).unwrap().data;
            let a3 = client.audit_read(r1_3, read.clone()).unwrap().data;

            let res2 = client
                .audit_list(
                    AuditListRequestBuilder::default()
                        .ge(Some(a1.created_at))
                        .le(Some(a3.created_at))
                        .offset_id(Some(a1.id))
                        .limit(Some(limit))
                        .build()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(res2.data.len(), 2);
            let r2_2 = res2.data[0].id;
            let r2_3 = res2.data[1].id;
            assert_eq!(r2_2, r1_2);
            assert_eq!(r2_3, r1_3);
        }

        #[test]
        #[ignore]
        fn api_audit_read_not_found_does_not_exist() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let client = client_create(Some(&service_key.value));

            let res = client
                .audit_read(Uuid::nil(), AuditReadRequest::new(None))
                .unwrap_err();
            assert_eq!(res.status_code(), StatusCode::NOT_FOUND.as_u16());
        }

        #[test]
        #[ignore]
        fn api_audit_read_not_found_masked_by_service() {
            let client = client_create(None);
            let (_service1, service_key1) = service_key_create(&client);
            let (_service2, service_key2) = service_key_create(&client);

            let client = client_create(Some(&service_key1.value));
            let a1 = client
                .audit_create(
                    AuditCreateRequestBuilder::default()
                        .type_("type_1".to_owned())
                        .build()
                        .unwrap(),
                )
                .unwrap()
                .data;

            let client = client_create(Some(&service_key2.value));
            let res = client
                .audit_read(a1.id, AuditReadRequest::new(None))
                .unwrap_err();
            assert_eq!(res.status_code(), StatusCode::NOT_FOUND.as_u16());
        }

        #[test]
        #[ignore]
        fn api_audit_read_unauthorised() {
            let client = client_create(Some(INVALID_KEY));
            let res = client
                .audit_read(Uuid::nil(), AuditReadRequest::new(None))
                .unwrap_err();
            assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED.as_u16());
        }

        #[test]
        #[ignore]
        fn api_audit_read_not_found_service_mask() {
            let client = client_create(None);
            let (_service1, service1_key) = service_key_create(&client);
            let (_service2, service2_key) = service_key_create(&client);

            let client = client_create(Some(&service1_key.value));
            let audit = client
                .audit_create(
                    AuditCreateRequestBuilder::default()
                        .type_("read_test".to_owned())
                        .build()
                        .unwrap(),
                )
                .unwrap()
                .data;

            let client = client_create(Some(&service2_key.value));
            let res = client
                .audit_read(audit.id, AuditReadRequest::new(None))
                .unwrap_err();
            assert_eq!(res.status_code(), StatusCode::NOT_FOUND.as_u16());
        }

        #[test]
        #[ignore]
        fn api_audit_read_ok() {
            let client = client_create(None);
            let audit = client
                .audit_create(
                    AuditCreateRequestBuilder::default()
                        .type_("read_test".to_owned())
                        .build()
                        .unwrap(),
                )
                .unwrap()
                .data;
            let res = client
                .audit_read(audit.id, AuditReadRequest::new(None))
                .unwrap();
            assert_eq!(res.data.id, audit.id);
        }

        #[test]
        #[ignore]
        fn api_audit_update_bad_request_service_mask() {
            let client = client_create(None);
            let (_service1, service1_key) = service_key_create(&client);
            let (_service2, service2_key) = service_key_create(&client);

            let client = client_create(Some(&service1_key.value));
            let audit = client
                .audit_create(
                    AuditCreateRequestBuilder::default()
                        .type_("update_test".to_owned())
                        .build()
                        .unwrap(),
                )
                .unwrap()
                .data;

            let client = client_create(Some(&service2_key.value));
            let res = client
                .audit_update(audit.id, AuditUpdateRequest::default().subject("example"))
                .unwrap_err();
            assert_eq!(res.status_code(), StatusCode::BAD_REQUEST.as_u16());
        }

        #[test]
        #[ignore]
        fn api_audit_update_ok() {
            let client = client_create(None);
            let (service1, service1_key) = service_key_create(&client);

            let client = client_create(Some(&service1_key.value));
            let audit1 = client
                .audit_create(
                    AuditCreateRequestBuilder::default()
                        .type_("update_test".to_owned())
                        .build()
                        .unwrap(),
                )
                .unwrap()
                .data;

            let audit2 = client
                .audit_update(
                    audit1.id,
                    AuditUpdateRequest::default()
                        .status_code(200)
                        .subject("example"),
                )
                .unwrap()
                .data;
            assert_eq!(audit1.id, audit2.id);
            assert_eq!(audit1.service_id, Some(service1.id));
            assert_eq!(audit1.status_code, None);
            assert_eq!(audit2.status_code, Some(200));
            assert_eq!(audit1.subject, None);
            assert_eq!(audit2.subject, Some("example".to_owned()));
        }
    };
}
