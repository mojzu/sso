use crate::{cli::Cli, internal::*};

impl Cli {
    pub async fn audit_read(config: &Config, id: Option<i64>) {
        let postgres = Postgres::from_config(config).await.unwrap();

        match id {
            Some(id) => match postgres.audit_read_id(id).await.unwrap() {
                Some(audit) => println!("{}", toml::to_string_pretty(&audit).unwrap()),
                None => println!("Audit ID not found"),
            },
            None => println!("No read arguments found"),
        }
    }
}
