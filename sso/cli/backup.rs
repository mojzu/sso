use crate::internal::*;

#[derive(Debug, Clone, Serialize)]
struct BackupPostgres {
    user: Option<String>,
    password: Option<String>,
    dbname: Option<String>,
    host: Option<String>,
    port: Option<u16>,
}

#[derive(Debug, Clone, Serialize)]
struct Backup {
    log: ConfigLog,
    http: ConfigHttp,
    oauth2: ConfigOauth2,
    postgres: BackupPostgres,
    mailto: mailto::Config,
    metrics: metrics::Config,
}

/// Backup configuration
pub async fn backup(config: &Config) {
    use tokio::fs::File;
    use tokio::prelude::*;

    let mut file = File::create("sso_config.backup.toml").await.unwrap();
    let backup = toml::to_string_pretty(&Backup {
        log: config.log.clone(),
        http: config.http.clone(),
        oauth2: config.oauth2.clone(),
        postgres: BackupPostgres {
            user: config.postgres.user.clone(),
            password: config.postgres.password.clone(),
            dbname: config.postgres.dbname.clone(),
            host: config.postgres.host.clone(),
            port: config.postgres.port.clone(),
        },
        mailto: config.mailto.clone(),
        metrics: config.metrics.clone(),
    })
    .unwrap();
    file.write_all(backup.as_bytes()).await.unwrap();
    println!("Backup complete");
}
