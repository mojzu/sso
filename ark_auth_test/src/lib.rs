pub mod support;
pub mod test_key;

#[macro_export]
macro_rules! integration_test {
    ($driver:expr) => {
        #[test]
        fn key_list_authorisation_test() {
            let (_, mut app) = $crate::support::app($driver);
            $crate::test_key::list_authorisation_test(&mut app)
            // TODO(refactor): Refactor integration tests here.
        }
    };
}
