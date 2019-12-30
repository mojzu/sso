#[macro_export]
macro_rules! audit_integration_test {
    () => {
        #[test]
        #[ignore]
        fn audit_list_created_ok() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);
            let mut client = client_create(Some(&service_key.value));
            let type_ = "test_1".to_owned();
            let limit = 3;

            let a1 = client
                .audit_create(pb::AuditCreateRequest::new(type_.clone()))
                .unwrap()
                .into_inner()
                .data
                .unwrap();
            client
                .audit_create(pb::AuditCreateRequest::new(type_.clone()))
                .unwrap();
            client
                .audit_create(pb::AuditCreateRequest::new(type_.clone()))
                .unwrap();
            client
                .audit_create(pb::AuditCreateRequest::new(type_.clone()))
                .unwrap();
            client
                .audit_create(pb::AuditCreateRequest::new(type_.clone()))
                .unwrap();

            let res1 = client
                .audit_list(pb::AuditListRequest::ge_limit(a1.created_at, limit))
                .unwrap()
                .into_inner();
            assert_eq!(res1.data.len(), 3);
            let r1_1 = res1.data[0].id.clone();
            let r1_2 = res1.data[1].id.clone();
            let r1_3 = res1.data[2].id.clone();
            assert_eq!(r1_1, a1.id);
            let a1 = client
                .audit_read(pb::AuditReadRequest {
                    id: r1_1.clone(),
                    subject: None,
                })
                .unwrap()
                .into_inner()
                .data
                .unwrap();

            let res2 = client
                .audit_list(pb::AuditListRequest::ge_offset_limit(
                    a1.created_at,
                    a1.id,
                    limit,
                ))
                .unwrap()
                .into_inner();
            assert_eq!(res2.data.len(), 3);
            let r2_2 = res2.data[0].id.clone();
            let r2_3 = res2.data[1].id.clone();
            let r2_4 = res2.data[2].id.clone();
            assert_eq!(r2_2, r1_2);
            assert_eq!(r2_3, r1_3);
            let a2 = client
                .audit_read(pb::AuditReadRequest {
                    id: r2_2.clone(),
                    subject: None,
                })
                .unwrap()
                .into_inner()
                .data
                .unwrap();

            let res3 = client
                .audit_list(pb::AuditListRequest::ge_offset_limit(
                    a2.created_at,
                    a2.id,
                    limit,
                ))
                .unwrap()
                .into_inner();
            assert_eq!(res3.data.len(), 3);
            let r3_3 = res3.data[0].id.clone();
            let r3_4 = res3.data[1].id.clone();
            let r3_5 = res3.data[2].id.clone();
            assert_eq!(r3_3, r2_3);
            assert_eq!(r3_4, r2_4);
            let a5 = client
                .audit_read(pb::AuditReadRequest {
                    id: r3_5.clone(),
                    subject: None,
                })
                .unwrap()
                .into_inner()
                .data
                .unwrap();

            let res4 = client
                .audit_list(pb::AuditListRequest::le_offset_limit(
                    a5.created_at,
                    a5.id,
                    limit,
                ))
                .unwrap()
                .into_inner();
            assert_eq!(res4.data.len(), 3);
            let r4_2 = res4.data[0].id.clone();
            let r4_3 = res4.data[1].id.clone();
            let r4_4 = res4.data[2].id.clone();
            assert_eq!(r4_2, r2_2);
            assert_eq!(r4_3, r3_3);
            assert_eq!(r4_4, r3_4);
            let a4 = client
                .audit_read(pb::AuditReadRequest {
                    id: r4_4.clone(),
                    subject: None,
                })
                .unwrap()
                .into_inner()
                .data
                .unwrap();

            let res5 = client
                .audit_list(pb::AuditListRequest::le_offset_limit(
                    a4.created_at,
                    a4.id,
                    limit,
                ))
                .unwrap()
                .into_inner();
            assert_eq!(res5.data.len(), 3);
            let r5_1 = res5.data[0].id.clone();
            let r5_2 = res5.data[1].id.clone();
            let r5_3 = res5.data[2].id.clone();
            assert_eq!(r5_1, r1_1);
            assert_eq!(r5_2, r4_2);
            assert_eq!(r5_3, r4_3);
        }

        #[test]
        #[ignore]
        fn audit_list_created_and_ok() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);
            let mut client = client_create(Some(&service_key.value));
            let type_ = "test_1".to_owned();
            let limit = 3;

            let a1 = client
                .audit_create(pb::AuditCreateRequest::new(type_.clone()))
                .unwrap()
                .into_inner()
                .data
                .unwrap();
            client
                .audit_create(pb::AuditCreateRequest::new(type_.clone()))
                .unwrap();
            client
                .audit_create(pb::AuditCreateRequest::new(type_.clone()))
                .unwrap();

            let res1 = client
                .audit_list(pb::AuditListRequest::ge_limit(a1.created_at, limit))
                .unwrap()
                .into_inner();
            assert_eq!(res1.data.len(), 3);
            let r1_1 = res1.data[0].id.clone();
            let r1_2 = res1.data[1].id.clone();
            let r1_3 = res1.data[2].id.clone();
            assert_eq!(r1_1, a1.id);
            let a1 = client
                .audit_read(pb::AuditReadRequest {
                    id: r1_1.clone(),
                    subject: None,
                })
                .unwrap()
                .into_inner()
                .data
                .unwrap();
            let a3 = client
                .audit_read(pb::AuditReadRequest {
                    id: r1_3.clone(),
                    subject: None,
                })
                .unwrap()
                .into_inner()
                .data
                .unwrap();

            let res2 = client
                .audit_list(pb::AuditListRequest::ge_le_offset_limit(
                    a1.created_at,
                    a3.created_at,
                    a1.id,
                    limit,
                ))
                .unwrap()
                .into_inner();
            assert_eq!(res2.data.len(), 2);
            let r2_2 = res2.data[0].id.clone();
            let r2_3 = res2.data[1].id.clone();
            assert_eq!(r2_2, r1_2);
            assert_eq!(r2_3, r1_3);
        }

        #[test]
        #[ignore]
        fn audit_read_not_found_does_not_exist() {
            let mut client = client_create(None);
            let (_service, service_key) = service_key_create(&mut client);
            let mut client = client_create(Some(&service_key.value));

            let res = client
                .audit_read(pb::AuditReadRequest {
                    id: UUID_NIL.to_owned(),
                    subject: None,
                })
                .unwrap_err();
            assert_eq!(res.code(), tonic::Code::NotFound);
        }

        #[test]
        #[ignore]
        fn audit_read_not_found_masked_by_service() {
            let mut client = client_create(None);
            let (_service1, service_key1) = service_key_create(&mut client);
            let (_service2, service_key2) = service_key_create(&mut client);

            let mut client = client_create(Some(&service_key1.value));
            let a1 = client
                .audit_create(pb::AuditCreateRequest::new("type_1".to_owned()))
                .unwrap()
                .into_inner()
                .data
                .unwrap();

            let mut client = client_create(Some(&service_key2.value));
            let res = client
                .audit_read(pb::AuditReadRequest {
                    id: a1.id,
                    subject: None,
                })
                .unwrap_err();
            assert_eq!(res.code(), tonic::Code::NotFound);
        }

        #[test]
        #[ignore]
        fn audit_read_unauthorised() {
            let mut client = client_create(Some(INVALID_KEY));
            let res = client
                .audit_read(pb::AuditReadRequest {
                    id: UUID_NIL.to_owned(),
                    subject: None,
                })
                .unwrap_err();
            assert_eq!(res.code(), tonic::Code::Unauthenticated);
        }

        #[test]
        #[ignore]
        fn audit_read_not_found_service_mask() {
            let mut client = client_create(None);
            let (_service1, service1_key) = service_key_create(&mut client);
            let (_service2, service2_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service1_key.value));
            let audit = client
                .audit_create(pb::AuditCreateRequest::new("read_test".to_owned()))
                .unwrap()
                .into_inner()
                .data
                .unwrap();

            let mut client = client_create(Some(&service2_key.value));
            let res = client
                .audit_read(pb::AuditReadRequest {
                    id: audit.id,
                    subject: None,
                })
                .unwrap_err();
            assert_eq!(res.code(), tonic::Code::NotFound);
        }

        #[test]
        #[ignore]
        fn audit_read_ok() {
            let mut client = client_create(None);
            let audit = client
                .audit_create(pb::AuditCreateRequest::new("read_test".to_owned()))
                .unwrap()
                .into_inner()
                .data
                .unwrap();
            let res = client
                .audit_read(pb::AuditReadRequest {
                    id: audit.id.clone(),
                    subject: None,
                })
                .unwrap()
                .into_inner()
                .data
                .unwrap();
            assert_eq!(res.id, audit.id);
        }

        #[test]
        #[ignore]
        fn audit_update_bad_request_service_mask() {
            let mut client = client_create(None);
            let (_service1, service1_key) = service_key_create(&mut client);
            let (_service2, service2_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service1_key.value));
            let audit = client
                .audit_create(pb::AuditCreateRequest::new("update_test".to_owned()))
                .unwrap()
                .into_inner()
                .data
                .unwrap();

            let mut client = client_create(Some(&service2_key.value));
            let res = client
                .audit_update(pb::AuditUpdateRequest {
                    id: audit.id,
                    status_code: None,
                    subject: Some("example".to_owned()),
                    data: None,
                })
                .unwrap_err();
            assert_eq!(res.code(), tonic::Code::InvalidArgument);
        }

        #[test]
        #[ignore]
        fn audit_update_ok() {
            let mut client = client_create(None);
            let (service1, service1_key) = service_key_create(&mut client);

            let mut client = client_create(Some(&service1_key.value));
            let audit1 = client
                .audit_create(pb::AuditCreateRequest::new("update_test".to_owned()))
                .unwrap()
                .into_inner()
                .data
                .unwrap();

            let audit2 = client
                .audit_update(pb::AuditUpdateRequest {
                    id: audit1.id.clone(),
                    status_code: Some(200),
                    subject: Some("example".to_owned()),
                    data: None,
                })
                .unwrap()
                .into_inner()
                .data
                .unwrap();
            assert_eq!(audit1.id, audit2.id);
            assert_eq!(audit1.service_id, Some(service1.id));
            assert_eq!(audit1.status_code, None);
            assert_eq!(audit2.status_code, Some(200));
            assert_eq!(audit1.subject, None);
            assert_eq!(audit2.subject, Some("example".to_owned()));
        }
    };
}
