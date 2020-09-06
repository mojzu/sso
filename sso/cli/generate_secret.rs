use crate::internal::*;

pub async fn generate_secret(config: &Config) {
    let postgres = Postgres::from_config(config).await.unwrap();
    let secret = postgres.secret_generate().await.unwrap();

    println!("{}", secret);
}
