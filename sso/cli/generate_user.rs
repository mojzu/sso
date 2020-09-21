use crate::internal::*;

#[derive(Debug, Serialize)]
struct Oauth2Wrapper {
    oauth2: UsersWrapper,
}

#[derive(Debug, Serialize)]
struct UsersWrapper {
    users: HashMap<String, ConfigOauth2User>,
}

/// Generate user configuration
pub async fn generate_user(
    config: &Config,
    user_id: Option<&str>,
    user_name: &str,
    user_email: &str,
) {
    let postgres = Postgres::from_config(config).await.unwrap();
    let password = postgres.secret_generate().await.unwrap();
    let user_password = postgres.password_hash(&password).await.unwrap();

    let user_id = match user_id {
        Some(user_id) => user_id.to_string(),
        None => Uuid::new_v4().to_string(),
    };

    let mut access = HashMap::new();
    for (id, _client) in config.oauth2.clients.iter() {
        access.insert(
            *id,
            ConfigOauth2UserAccess {
                enable: true,
                scope: Vec::new(),
            },
        );
    }

    let value = ConfigOauth2User {
        name: user_name.to_string(),
        email: user_email.to_string(),
        password: Some(user_password),
        locale: "".to_string(),
        timezone: "".to_string(),
        enable: true,
        access,
    };

    let mut users = HashMap::new();
    users.insert(user_id, value);

    let wrapped = Oauth2Wrapper {
        oauth2: UsersWrapper { users },
    };

    println!("The user password is:\r\n");
    println!("{}\r\n", password);

    println!("Add the following to the configuration file:\r\n");
    println!("{}", toml::to_string_pretty(&wrapped).unwrap());
}
