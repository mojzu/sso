use ark_auth::{driver, driver::Driver};
use ark_auth_test::integration_test;

fn initialise_driver() -> Box<Driver> {
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let driver = driver::postgres::Driver::initialise(&database_url, 1).unwrap();
    driver.box_clone()
}

integration_test!(initialise_driver());
