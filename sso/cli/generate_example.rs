use crate::{cli::Cli, internal::*};

impl Cli {
    pub async fn generate_example(config: &Config) {
        let postgres = Postgres::from_config(config).await.unwrap();

        // Todo: Implement this
        unimplemented!();
    }
}
