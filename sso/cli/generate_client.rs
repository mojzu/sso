use crate::internal::*;

#[derive(Debug, Serialize)]
struct Oauth2Wrapper {
    oauth2: ClientsWrapper,
}

#[derive(Debug, Serialize)]
struct ClientsWrapper {
    clients: HashMap<String, ConfigOauth2Client>,
}

pub async fn generate_client(
    config: &Config,
    client_id: Option<&str>,
    client_name: &str,
    redirect_uri: &str,
    scope: &str,
) {
    let postgres = Postgres::from_config(config).await.unwrap();
    let secret = postgres.secret_generate().await.unwrap();

    let client_id = match client_id {
        Some(client_id) => client_id.to_string(),
        None => Uuid::new_v4().to_string(),
    };
    let client_secret = postgres.secret_hash(&secret, &client_id).await.unwrap();
    let scope = oauth2::Scope::from_string(scope);

    let value = ConfigOauth2Client {
        name: client_name.to_string(),
        uri: Url::parse(redirect_uri).unwrap(),
        secret: client_secret.to_string(),
        redirect_uri: Url::parse(redirect_uri).unwrap(),
        enable: true,
        scope: scope.into_inner(),
        user_scope: oauth2::Scope::default().into_inner(),
        register_enable: true,
        register_scope: oauth2::Scope::default().into_inner(),
        ttl: ConfigOauth2ClientTtl::default(),
        templates: ConfigOauth2ClientTemplates::default(),
    };

    let mut clients = HashMap::new();
    clients.insert(client_id.clone(), value);

    let wrapped = Oauth2Wrapper {
        oauth2: ClientsWrapper { clients },
    };

    println!("The client ID key is:\r\n");
    println!("{}\r\n", client_id);

    println!("The client secret key is:\r\n");
    println!("{}\r\n", secret);

    println!("Add the following to the configuration file:\r\n");
    println!("{}", toml::to_string_pretty(&wrapped).unwrap());
}
