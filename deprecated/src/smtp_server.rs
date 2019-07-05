// TODO(test): SMTP testing using mailin_embedded.
// pub fn password_confirm_test(driver: &Driver, app: &mut TestServerRuntime) {
//     let (service, key) = support::service_key(driver);
//     let (_servic2, key2) = support::service_key(driver);
//     let (user, _key) = support::user_key(driver, &service, Some("guest"));
//     let (_, token) = core::auth::reset_password(driver, &service, &user.email).unwrap();
//     // Service 2 cannot confirm reset password.
//     // 400 BAD REQUEST response.
//     let payload = format!(
//         r#"{{"token": "{}", "password": "guestguest"}}"#,
//         &token.token
//     );
//     let (status_code, content_length, bytes) = support::app_post(
//         app,
//         "/v1/auth/provider/local/reset/password/confirm",
//         Some(&key2.value),
//         payload,
//     );
//     assert_eq!(status_code, StatusCode::BAD_REQUEST);
//     assert_eq!(content_length, 0);
//     assert_eq!(bytes.len(), 0);
//     // Confirm reset password success.
//     // 200 OK response.
//     let payload = format!(
//         r#"{{"token": "{}", "password": "guestguest"}}"#,
//         &token.token
//     );
//     let (status_code, content_length, bytes) = support::app_post(
//         app,
//         "/v1/auth/provider/local/reset/password/confirm",
//         Some(&key.value),
//         payload,
//     );
//     assert_eq!(status_code, StatusCode::OK);
//     assert_eq!(content_length, bytes.len());
//     let body: server::route::auth::reset::PasswordConfirmResponse =
//         serde_json::from_slice(&bytes).unwrap();
//     assert!(body.meta.password_strength.is_some());
//     assert_eq!(body.meta.password_pwned, None);
//     // User password is updated.
//     // 200 OK response.
//     let payload = format!(
//         r#"{{"email": "{}", "password": "guestguest"}}"#,
//         &user.email
//     );
//     let (status_code, _content_length, _bytes) =
//         support::app_post(app, "/v1/auth/provider/local/login", Some(&key.value), payload);
//     assert_eq!(status_code, StatusCode::OK);
//     // Cannot reuse token.
//     // 400 BAD REQUEST response.
//     let payload = format!(r#"{{"token": "{}", "password": "guest"}}"#, &token.token);
//     let (status_code, content_length, bytes) = support::app_post(
//         app,
//         "/v1/auth/provider/local/reset/password/confirm",
//         Some(&key.value),
//         payload,
//     );
//     assert_eq!(status_code, StatusCode::BAD_REQUEST);
//     assert_eq!(content_length, 0);
//     assert_eq!(bytes.len(), 0);
// }
