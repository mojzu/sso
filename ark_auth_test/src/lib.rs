#[macro_use]
extern crate serde_derive;

pub mod support;
pub mod test_auth;
pub mod test_auth_key;
pub mod test_auth_reset;
pub mod test_auth_token;
pub mod test_key;
pub mod test_service;
pub mod test_user;

// TODO(refactor): Refactor integration tests here.

#[macro_export]
macro_rules! integration_test {
    ($driver:expr) => {
        // Authentication key tests.
        #[test]
        fn auth_key_verify_authorisation_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_key::verify_authorisation_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_key_verify_body_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_key::verify_body_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_key_verify_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_key::verify_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_key_revoke_authorisation_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_key::revoke_authorisation_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_key_revoke_body_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_key::revoke_body_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_key_revoke_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_key::revoke_test(drv.as_ref(), &mut app)
        }

        // Authentication reset tests.
        #[test]
        fn auth_reset_password_authorisation_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_reset::password_authorisation_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_reset_password_body_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_reset::password_body_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_reset_password_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_reset::password_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_reset_password_confirm_authorisation_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_reset::password_confirm_authorisation_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_reset_password_confirm_body_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_reset::password_confirm_body_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_reset_password_confirm_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_reset::password_confirm_test(drv.as_ref(), &mut app)
        }

        // Authentication token tests.
        #[test]
        fn auth_token_verify_authorisation_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_token::verify_authorisation_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_token_verify_body_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_token::verify_body_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_token_verify_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_token::verify_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_token_refresh_authorisation_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_token::refresh_authorisation_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_token_refresh_body_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_token::refresh_body_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_token_refresh_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_token::refresh_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_token_revoke_authorisation_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_token::revoke_authorisation_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_token_revoke_body_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_token::revoke_body_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_token_revoke_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth_token::revoke_test(drv.as_ref(), &mut app)
        }

        // Authentication tests.
        #[test]
        fn auth_login_authorisation_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth::login_authorisation_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_login_body_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth::login_body_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn auth_login_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_auth::login_test(drv.as_ref(), &mut app)
        }

        // Key tests.
        #[test]
        fn key_list_authorisation_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_key::list_authorisation_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn key_list_query_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_key::list_query_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn key_list_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_key::list_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn key_create_authorisation_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_key::create_authorisation_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn key_read_authorisation_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_key::read_authorisation_test(drv.as_ref(), &mut app)
        }

        // Service tests.
        #[test]
        fn service_read_authorisation_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_service::read_authorisation_test(drv.as_ref(), &mut app)
        }

        // User tests.
        #[test]
        fn user_list_authorisation_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_user::list_authorisation_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn user_create_authorisation_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_user::create_authorisation_test(drv.as_ref(), &mut app)
        }
        #[test]
        fn user_create_test() {
            let (drv, mut app) = $crate::support::app($driver);
            $crate::test_user::create_test(drv.as_ref(), &mut app)
        }
    };
}
