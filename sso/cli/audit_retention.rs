use crate::{cli::Cli, internal::*};

impl Cli {
    pub async fn audit_retention(config: &Config, days: i32) {
        let postgres = Postgres::from_config(config).await.unwrap();

        let deleted = postgres.audit_retention(days).await.unwrap();
        println!("Deleted {} audit logs", deleted);
    }
}
