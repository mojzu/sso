use crate::internal::*;

/// Teardown postgres database
pub async fn postgres_teardown(config: &Config) {
    let postgres = Postgres::from_config(config).await.unwrap();

    postgres.teardown().await.unwrap();

    println!("Teardown complete");
}
