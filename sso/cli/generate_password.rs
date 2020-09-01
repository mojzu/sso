use crate::{cli::Cli, internal::*};

impl Cli {
    pub async fn generate_password(config: &Config) {
        let postgres = Postgres::from_config(config).await.unwrap();
        let password = postgres.secret_generate().await.unwrap();
        let password_hash = postgres.password_hash(&password).await.unwrap();

        println!("The password is:\r\n");
        println!("{}\r\n", password);

        println!("The password hash is:\r\n");
        println!("{}\r\n", password_hash);
    }
}
