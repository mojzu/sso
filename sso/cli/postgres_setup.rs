use crate::internal::*;

pub async fn postgres_setup(config: &Config) {
    let _ = Postgres::from_config(config).await.unwrap();

    println!("Setup complete");
}
