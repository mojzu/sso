use crate::internal::*;
use tokio_postgres::Row;

/// Postgres
#[derive(Clone)]
pub(crate) struct Postgres {
    pool: deadpool_postgres::Pool,
}

/// Postgres Query
struct PostgresQuery;

impl Postgres {
    /// Create postgres database connection pool using configuration and run setup
    pub async fn from_config(config: &Config) -> Result<Self> {
        let pool = config.postgres.create_pool(tokio_postgres::NoTls)?;

        Self::setup(&pool, &config.oauth2.users).await?;

        Ok(Self { pool })
    }

    /// Returns ok if postgres connection can execute queries
    pub async fn readiness(&self) -> Result<()> {
        let conn = self.pool.get().await?;
        PostgresQuery::readiness(&conn).await
    }

    /// Run idempotent setup script to initialise database, prepare queries and write
    /// necessary static configuration into database
    async fn setup(
        pool: &deadpool_postgres::Pool,
        users: &HashMap<Uuid, ConfigOauth2User>,
    ) -> Result<()> {
        let conn = pool.get().await?;
        conn.batch_execute(include_str!("setup.sql")).await?;

        let mut exclude_user_id: Vec<Uuid> = Vec::new();
        for (user_id, user) in users.iter() {
            exclude_user_id.push(*user_id);

            PostgresQuery::user_upsert_static(&conn, user_id, user).await?;

            if let Some(password) = user.password.as_ref() {
                PostgresQuery::user_password_upsert(&conn, user_id, password, false, false, true)
                    .await?;
            }

            for (client_id, access) in user.access.iter() {
                let scope = oauth2::Scope::from_ref(&access.scope);
                PostgresQuery::access_upsert(
                    &conn,
                    client_id,
                    user_id,
                    access.enable,
                    &scope,
                    true,
                )
                .await?;
            }
        }

        PostgresQuery::user_delete_static(&conn, &exclude_user_id).await?;

        Ok(())
    }

    pub async fn teardown(&self) -> Result<()> {
        let conn = self.pool.get().await?;
        conn.batch_execute(include_str!("teardown.sql")).await?;
        Ok(())
    }

    pub async fn secret_generate(&self) -> Result<String> {
        let conn = self.pool.get().await?;
        PostgresQuery::secret_generate(&conn, 32).await
    }

    pub async fn secret_hash(&self, secret: &str, value: &str) -> Result<String> {
        let conn = self.pool.get().await?;
        PostgresQuery::secret_hash(&conn, secret, value).await
    }

    pub async fn password_hash(&self, password: &str) -> Result<String> {
        let client = self.pool.get().await?;

        let statement = client.prepare("SELECT sso._password_hash($1)").await?;

        let rows = client.query(&statement, &[&password.to_string()]).await?;
        let value: &str = rows[0].get(0);

        Ok(value.to_string())
    }

    pub async fn password_check(
        &self,
        email: &str,
        password: &str,
    ) -> Result<PostgresUserPasswordCheck> {
        let conn = self.pool.get().await?;

        let statement = conn
            .prepare(include_str!("user_password_check.sql"))
            .await?;

        let mut rows = conn.query(&statement, &[&email, &password]).await?;

        if rows.is_empty() {
            return Err("email not found".into());
        }
        let row: PostgresUserPasswordCheck = rows.remove(0).into();

        if !row.enable {
            return Err("user is disabled".into());
        }
        if !row.check {
            return Err("password is incorrect".into());
        }

        Ok(row)
    }

    pub async fn user_read(
        &self,
        client: &Client,
        req: RequestUserRead,
    ) -> Result<ResponseUserMany> {
        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("user_read.sql")).await?;

        let rows = conn
            .query(&statement, &[&client.client_id, &req.id, &req.email])
            .await?;

        Ok(ResponseUserMany {
            data: rows.into_iter().map(|x| x.into()).collect(),
        })
    }

    pub async fn user_update(
        &self,
        client: &Client,
        req: RequestUserUpdate,
    ) -> Result<ResponseUser> {
        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("user_update.sql")).await?;

        let count = conn
            .execute(
                &statement,
                &[
                    &req.id,
                    &req.name,
                    &req.email,
                    &req.locale,
                    &req.timezone,
                    &req.enable,
                ],
            )
            .await?;

        if count == 1 {
            if let Some(password) = req.password {
                self.user_password_update2(req.id, password).await?;
            }
            if let Some(access) = req.access {
                self.user_access_update(client, req.id, access).await?;
            }

            let mut user = self
                .user_read(
                    client,
                    RequestUserRead {
                        id: Some(vec![req.id]),
                        email: None,
                    },
                )
                .await?;
            Ok(user.data.remove(0))
        } else {
            Err(Error::from("user not found"))
        }
    }

    pub async fn user_access_read(
        &self,
        client: &Client,
        req: RequestUserAccessRead,
    ) -> Result<ResponseAccess> {
        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("user_access_read.sql")).await?;
        let mut rows = conn
            .query(&statement, &[&client.client_id, &req.user_id])
            .await?;

        if rows.is_empty() {
            return Err(Error::from("access not found"));
        }
        let row: ResponseAccess = rows.remove(0).into();
        Ok(row)
    }

    pub async fn user_access_insert(
        &self,
        client: &Client,
        req: RequestUserCreate,
    ) -> Result<ResponseUser> {
        let user_id = Uuid::new_v4();
        let scope = oauth2::Scope::from_string(&req.scope);
        if !client.user_scope.contains(&scope) {
            return Err(Error::from("scope invalid"));
        }

        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("user_insert.sql")).await?;

        let rows = conn
            .query(
                &statement,
                &[
                    &user_id,
                    &req.name,
                    &req.email,
                    &req.locale,
                    &req.timezone,
                    &req.enable,
                ],
            )
            .await?;

        let password = if let Some(password) = &req.password {
            let res = self
                .user_password_upsert(
                    user_id,
                    &password.password,
                    password.allow_reset,
                    password.require_update,
                )
                .await?;
            Some(res)
        } else {
            None
        };

        let access = self
            .access_upsert(
                client,
                RequestAccessUpdate {
                    user_id,
                    enable: true,
                    scope: scope.to_string(),
                },
            )
            .await?;

        Ok(ResponseUser {
            id: rows[0].get("id"),
            created_at: rows[0].get("created_at"),
            updated_at: rows[0].get("updated_at"),
            name: rows[0].get("name"),
            email: rows[0].get("email"),
            locale: rows[0].get("locale"),
            timezone: rows[0].get("timezone"),
            enable: rows[0].get("enable"),
            static_: rows[0].get("static"),
            password,
            oauth2_provider: Vec::new(),
            oauth2_provider_count: 0,
            access: Some(access),
        })
    }

    async fn user_upsert_email(&self, name: &str, email: &str) -> Result<Uuid> {
        let client = self.pool.get().await?;

        let statement = client
            .prepare(include_str!("user_upsert_email.sql"))
            .await?;

        let id = Uuid::new_v4();
        let rows = client.query(&statement, &[&id, &name, &email]).await?;
        let id: Uuid = rows[0].get("id");

        Ok(id)
    }

    pub async fn user_password_reset_accept(&self, id: Uuid, password: &str) -> Result<()> {
        let client = self.pool.get().await?;

        let statement = client
            .prepare(include_str!("user_password_reset_accept.sql"))
            .await?;

        client.query(&statement, &[&id, &password]).await?;
        Ok(())
    }

    pub async fn user_email_update(&self, id: Uuid, password: &str, email_new: &str) -> Result<()> {
        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("user_email_update.sql")).await?;

        let rows = conn
            .execute(&statement, &[&id, &password, &email_new])
            .await?;

        match rows {
            1 => Ok(()),
            _ => Err(Error::from("email update failed")),
        }
    }

    pub async fn user_password_update(
        &self,
        id: Uuid,
        password: &str,
        password_new: &str,
    ) -> Result<()> {
        let conn = self.pool.get().await?;

        let statement = conn
            .prepare(include_str!("user_password_update.sql"))
            .await?;

        let rows = conn
            .execute(&statement, &[&id, &password, &password_new])
            .await?;

        match rows {
            1 => Ok(()),
            _ => Err(Error::from("password update failed")),
        }
    }

    pub async fn user_register_accept_password(
        &self,
        client: &Client,
        scope: &oauth2::Scope,
        email: &str,
        name: &str,
        password: &str,
        password_allow_reset: bool,
    ) -> Result<Uuid> {
        if !client.user_scope.contains(&scope) {
            return Err(Error::from("scope invalid"));
        }

        let user_id = self.user_upsert_email(name, email).await?;

        self.user_password_upsert(user_id, password, password_allow_reset, false)
            .await?;

        self.access_upsert(
            client,
            RequestAccessUpdate {
                user_id,
                enable: true,
                scope: scope.to_string(),
            },
        )
        .await?;

        Ok(user_id)
    }

    async fn user_access_update(
        &self,
        client: &Client,
        user_id: Uuid,
        req: RequestUserAccessUpdate,
    ) -> Result<Option<ResponseAccess>> {
        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("user_access_update.sql")).await?;

        let rows = conn
            .query(
                &statement,
                &[&client.client_id, &user_id, &req.enable, &req.scope],
            )
            .await?;

        if !rows.is_empty() {
            Ok(Some(ResponseAccess {
                client_id: rows[0].get("client_id"),
                user_id: rows[0].get("user_id"),
                created_at: rows[0].get("created_at"),
                updated_at: rows[0].get("updated_at"),
                enable: rows[0].get("enable"),
                scope: rows[0].get("scope"),
                static_: rows[0].get("static"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn user_password_update2(
        &self,
        user_id: Uuid,
        req: RequestUserPasswordUpdate,
    ) -> Result<Option<ResponseUserPassword>> {
        let conn = self.pool.get().await?;

        let statement = conn
            .prepare(include_str!("user_password_update2.sql"))
            .await?;

        let rows = conn
            .query(
                &statement,
                &[&user_id, &req.allow_reset, &req.require_update],
            )
            .await?;

        if !rows.is_empty() {
            Ok(Some(ResponseUserPassword {
                user_id: rows[0].get("user_id"),
                created_at: rows[0].get("created_at"),
                updated_at: rows[0].get("updated_at"),
                allow_reset: rows[0].get("allow_reset"),
                require_update: rows[0].get("require_update"),
                static_: rows[0].get("static"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn user_password_upsert(
        &self,
        id: Uuid,
        password: &str,
        allow_reset: bool,
        require_update: bool,
    ) -> Result<ResponseUserPassword> {
        let hash = self.password_hash(password).await?;

        let conn = self.pool.get().await?;

        PostgresQuery::user_password_upsert(&conn, &id, &hash, allow_reset, require_update, false)
            .await
    }

    pub async fn user_register_accept_oauth2_provider(
        &self,
        client: &Client,
        scope: &oauth2::Scope,
        email: &str,
        name: &str,
        provider: PostgresOauth2Provider,
        sub: &str,
    ) -> Result<Uuid> {
        if !client.user_scope.contains(&scope) {
            return Err(Error::from("scope invalid"));
        }

        let user_id = self.user_upsert_email(&name, &email).await?;

        let conn = self.pool.get().await?;

        let statement = conn
            .prepare(include_str!("user_oauth2_provider_insert.sql"))
            .await?;

        conn.query(&statement, &[&user_id, &provider, &sub, &false])
            .await?;

        self.access_upsert(
            client,
            RequestAccessUpdate {
                user_id,
                enable: true,
                scope: scope.to_string(),
            },
        )
        .await?;

        Ok(user_id)
    }

    pub async fn user_delete(&self, user_id: Uuid) -> Result<()> {
        let client = self.pool.get().await?;

        let statement = client.prepare(include_str!("user_delete.sql")).await?;

        client.query(&statement, &[&user_id]).await?;
        Ok(())
    }

    pub async fn user_oauth2_provider_check(
        &self,
        provider: PostgresOauth2Provider,
        sub: &str,
    ) -> Result<Uuid> {
        let client = self.pool.get().await?;

        let statement = client
            .prepare(include_str!("user_oauth2_provider_check.sql"))
            .await?;

        let rows = client.query(&statement, &[&provider, &sub]).await?;
        if !rows.is_empty() {
            let user_id: Uuid = rows[0].get("user_id");
            Ok(user_id)
        } else {
            Err("access not found".into())
        }
    }

    pub async fn csrf_insert(&self, client: &Client) -> Result<ResponseCsrf> {
        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("csrf_insert.sql")).await?;

        let rows = conn
            .query(&statement, &[&client.client_id, &client.ttl.csrf_s])
            .await?;

        Ok(ResponseCsrf {
            client_id: rows[0].get("client_id"),
            token: rows[0].get("token"),
            created_at: rows[0].get("created_at"),
            ttl: rows[0].get("ttl"),
        })
    }

    pub async fn csrf_verify(&self, client: &Client, request: RequestCsrf) -> Result<()> {
        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("csrf_verify.sql")).await?;

        let rows = conn
            .query(&statement, &[&client.client_id, &request.token])
            .await?;
        if !rows.is_empty() {
            Ok(())
        } else {
            Err("csrf token not found or expired".into())
        }
    }

    pub async fn code_insert_auth(
        &self,
        client: &Client,
        ttl_s: i64,
        user_id: Uuid,
        state: &str,
        scope: &str,
    ) -> Result<String> {
        let scope = oauth2::Scope::from_string(scope);
        if !client.user_scope.contains(&scope) {
            return Err(Error::from("scope invalid"));
        }

        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("code_insert_auth.sql")).await?;

        let rows = conn
            .query(
                &statement,
                &[
                    &client.client_id,
                    &ttl_s,
                    &user_id,
                    &state,
                    &scope.to_string(),
                ],
            )
            .await?;
        let value: &str = rows[0].get(0);

        Ok(value.to_string())
    }

    pub async fn code_insert_password_reset(
        &self,
        client_id: Uuid,
        ttl_s: i64,
        email: &str,
    ) -> Result<String> {
        let client = self.pool.get().await?;

        let statement = client
            .prepare(include_str!("code_insert_password_reset.sql"))
            .await?;

        let rows = client
            .query(&statement, &[&client_id, &ttl_s, &email])
            .await?;
        if !rows.is_empty() {
            let value: String = rows[0].get("value");

            Ok(value)
        } else {
            Err("email not found".into())
        }
    }

    pub async fn code_insert_register(
        &self,
        client_id: Uuid,
        ttl_s: i64,
        email: &str,
    ) -> Result<String> {
        let client = self.pool.get().await?;

        let statement = client
            .prepare(include_str!("code_insert_register.sql"))
            .await?;

        let rows = client
            .query(&statement, &[&client_id, &ttl_s, &email])
            .await?;
        if !rows.is_empty() {
            let value: String = rows[0].get("value");

            Ok(value)
        } else {
            Err("email not found".into())
        }
    }

    pub async fn code_insert_delete(
        &self,
        client_id: Uuid,
        ttl_s: i64,
        user_id: Uuid,
        password: &str,
    ) -> Result<(String, String)> {
        let client = self.pool.get().await?;

        let statement = client
            .prepare(include_str!("code_insert_delete.sql"))
            .await?;

        let rows = client
            .query(&statement, &[&client_id, &ttl_s, &user_id, &password])
            .await?;
        if !rows.is_empty() {
            let value: String = rows[0].get(0);
            let email: String = rows[0].get(1);

            Ok((value, email))
        } else {
            Err("user not found or password incorrect".into())
        }
    }

    pub async fn code_verify(
        &self,
        client_id: Uuid,
        value: &str,
        target: PostgresCodeTarget,
    ) -> Result<PostgresCode> {
        let client = self.pool.get().await?;

        let statement = client.prepare(include_str!("code_verify.sql")).await?;

        let rows = client
            .query(&statement, &[&client_id, &value, &target])
            .await?;
        if !rows.is_empty() {
            let client_id: Uuid = rows[0].get("client_id");
            let user_id: Option<Uuid> = rows[0].get("user_id");
            let state: String = rows[0].get("state");
            let scope: String = rows[0].get("scope");
            let email: String = rows[0].get("email");

            Ok(PostgresCode {
                client_id,
                user_id,
                state,
                scope: scope.into(),
                email,
            })
        } else {
            Err("code not found or expired".into())
        }
    }

    pub async fn code_read_client(&self, code: &str) -> Result<Uuid> {
        let client = self.pool.get().await?;

        let statement = client.prepare(include_str!("code_read_client.sql")).await?;

        let rows = client.query(&statement, &[&code]).await?;
        if !rows.is_empty() {
            let client_id: Uuid = rows[0].get("client_id");
            Ok(client_id)
        } else {
            Err("code not found or expired".into())
        }
    }

    pub async fn oauth2_code_insert_auth(
        &self,
        client_id: Uuid,
        ttl_s: i64,
        provider: PostgresOauth2Provider,
        csrf: &str,
        pkce: Option<&str>,
        req: oauth2::AuthorizationCodeRequest,
    ) -> Result<String> {
        let client = self.pool.get().await?;

        let statement = client
            .prepare(include_str!("oauth2_code_insert_auth.sql"))
            .await?;

        let rows = client
            .query(
                &statement,
                &[
                    &client_id,
                    &csrf,
                    &provider,
                    &PostgresOauth2Target::Auth,
                    &ttl_s,
                    &pkce.unwrap_or(""),
                    &req.redirect_uri().to_string(),
                    &req.state(),
                    &req.scope().to_string(),
                ],
            )
            .await?;
        let csrf: String = rows[0].get("csrf");

        Ok(csrf)
    }

    pub async fn oauth2_code_insert_register(
        &self,
        client_id: Uuid,
        ttl_s: i64,
        provider: PostgresOauth2Provider,
        csrf: &str,
        pkce: Option<&str>,
        email: &str,
    ) -> Result<String> {
        let client = self.pool.get().await?;

        let statement = client
            .prepare(include_str!("oauth2_code_insert_register.sql"))
            .await?;

        let rows = client
            .query(
                &statement,
                &[
                    &client_id,
                    &csrf,
                    &provider,
                    &PostgresOauth2Target::Register,
                    &ttl_s,
                    &pkce.unwrap_or(""),
                    &email,
                ],
            )
            .await?;
        let csrf: String = rows[0].get("csrf");

        Ok(csrf)
    }

    pub async fn oauth2_code_read_client(&self, csrf: &str) -> Result<Uuid> {
        let client = self.pool.get().await?;

        let statement = client
            .prepare(include_str!("oauth2_code_read_client.sql"))
            .await?;

        let rows = client.query(&statement, &[&csrf]).await?;
        if !rows.is_empty() {
            let client_id: Uuid = rows[0].get("client_id");
            Ok(client_id)
        } else {
            Err("csrf not found or expired".into())
        }
    }

    pub async fn oauth2_code_verify(
        &self,
        client_id: Uuid,
        csrf: &str,
    ) -> Result<PostgresOauth2Code> {
        let client = self.pool.get().await?;

        let statement = client
            .prepare(include_str!("oauth2_code_verify.sql"))
            .await?;

        let rows = client.query(&statement, &[&client_id, &csrf]).await?;
        if !rows.is_empty() {
            let redirect_uri: &str = rows[0].get("redirect_uri");
            let redirect_uri = if !redirect_uri.is_empty() {
                Some(Url::parse(redirect_uri).unwrap())
            } else {
                None
            };

            Ok(PostgresOauth2Code {
                provider: rows[0].get("provider"),
                target: rows[0].get("target"),
                pkce: rows[0].get("pkce"),
                redirect_uri,
                state: rows[0].get("state"),
                scope: rows[0].get("scope"),
                email: rows[0].get("email"),
            })
        } else {
            Err("code not found or expired".into())
        }
    }

    pub async fn token_insert(
        &self,
        client: &Client,
        client_secret: &str,
        user_id: Uuid,
        ttl_s: i64,
        name: &str,
        scope: &oauth2::Scope,
    ) -> Result<PostgresToken> {
        if !client.user_scope.contains(scope) {
            return Err(Error::from("scope invalid"));
        }

        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("token_insert.sql")).await?;

        let id = Uuid::new_v4();
        let rows = conn
            .query(
                &statement,
                &[
                    &id,
                    &client.client_id,
                    &user_id,
                    &ttl_s,
                    &name,
                    &true,
                    &scope.to_string(),
                    &client_secret,
                ],
            )
            .await?;
        let access_token: String = rows[0].get(0);
        let refresh_token: String = rows[0].get(1);
        let scope: String = rows[0].get(2);

        Ok(PostgresToken {
            access_token,
            refresh_token,
            scope: scope.into(),
        })
    }

    pub async fn token_refresh(
        &self,
        client_id: Uuid,
        ttl_access_s: i64,
        ttl_refresh_s: i64,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<PostgresToken> {
        let client = self.pool.get().await?;

        let statement = client.prepare(include_str!("token_refresh.sql")).await?;

        let rows = client
            .query(
                &statement,
                &[
                    &client_id,
                    &ttl_access_s,
                    &ttl_refresh_s,
                    &client_secret,
                    &refresh_token,
                ],
            )
            .await?;

        if !rows.is_empty() {
            let scope: String = rows[0].get("scope");
            Ok(PostgresToken {
                access_token: rows[0].get("value"),
                refresh_token: rows[0].get("value_refresh"),
                scope: scope.into(),
            })
        } else {
            Err("token refresh failed".into())
        }
    }

    pub async fn token_introspect(
        &self,
        client_id: Uuid,
        ttl_refresh_s: i64,
        client_secret: &str,
        token: &str,
    ) -> Result<Option<oauth2::IntrospectionResponseArgs>> {
        let client = self.pool.get().await?;

        let statement = client.prepare(include_str!("token_introspect.sql")).await?;

        let rows = client
            .query(
                &statement,
                &[&client_id, &ttl_refresh_s, &client_secret, &token],
            )
            .await?;

        if !rows.is_empty() {
            let scope: String = rows[0].get("scope");
            let username: &str = rows[0].get("username");
            let sub: &str = rows[0].get("sub");
            Ok(Some(oauth2::IntrospectionResponseArgs::new(
                scope, username, sub,
            )))
        } else {
            Ok(None)
        }
    }

    pub async fn access_upsert(
        &self,
        client: &Client,
        req: RequestAccessUpdate,
    ) -> Result<ResponseAccess> {
        let scope = oauth2::Scope::from_string(req.scope);
        if !client.user_scope.contains(&scope) {
            return Err(Error::from("scope invalid"));
        }

        let conn = self.pool.get().await?;
        PostgresQuery::access_upsert(
            &conn,
            &client.client_id,
            &req.user_id,
            req.enable,
            &scope,
            false,
        )
        .await
    }

    pub async fn access_read(&self, client: &Client, user_id: Uuid) -> Result<oauth2::Scope> {
        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("access_read.sql")).await?;
        let mut rows = conn
            .query(&statement, &[&client.client_id, &user_id])
            .await?;

        if rows.is_empty() {
            return Err(Error::from("access not found"));
        }
        let row: AccessRead = rows.remove(0).into();

        if !row.enable {
            return Err(Error::from("access is disabled"));
        }
        Ok(oauth2::Scope::from_string(row.scope))
    }

    pub async fn access_read_many(
        &self,
        client: &Client,
        _req: RequestAccessRead,
    ) -> Result<ResponseAccessMany> {
        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("access_read_many.sql")).await?;
        let rows = conn.query(&statement, &[&client.client_id]).await?;

        let res = ResponseAccessMany {
            data: rows
                .into_iter()
                .map(|x| ResponseAccess {
                    client_id: x.get("client_id"),
                    user_id: x.get("user_id"),
                    created_at: x.get("created_at"),
                    updated_at: x.get("updated_at"),
                    enable: x.get("enable"),
                    scope: x.get("scope"),
                    static_: x.get("static"),
                })
                .collect(),
        };
        Ok(res)
    }

    pub async fn access_delete(&self, client: &Client, req: RequestAccessDelete) -> Result<()> {
        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("access_delete.sql")).await?;
        conn.query(&statement, &[&client.client_id, &req.user_id])
            .await?;

        Ok(())
    }

    pub async fn api_key_create(
        &self,
        client: &Client,
        req: RequestApiKeyCreate,
    ) -> Result<ResponseApiKey> {
        let id = Uuid::new_v4();
        let secret = self.secret_generate().await?;
        let secret_hash = self.secret_hash(&secret, &id.to_string()).await?;
        let scope = oauth2::Scope::from_string(req.scope);
        if !client.user_scope.contains(&scope) {
            return Err(Error::from("scope invalid"));
        }

        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("api_key_insert.sql")).await?;

        let row = conn
            .query_one(
                &statement,
                &[
                    &id,
                    &client.client_id,
                    &req.user_id,
                    &secret_hash,
                    &req.name,
                    &req.enable,
                    &scope.to_string(),
                ],
            )
            .await?;

        let mut key: ResponseApiKey = row.into();
        key.value = Some(format!("{}.{}", id, secret));

        Ok(key)
    }

    pub async fn api_key_read(
        &self,
        client: &Client,
        req: RequestApiKeyRead,
    ) -> Result<ResponseApiKeyMany> {
        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("api_key_read.sql")).await?;
        let rows = conn
            .query(&statement, &[&client.client_id, &req.id, &req.user_id])
            .await?;

        Ok(ResponseApiKeyMany {
            data: rows.into_iter().map(|x| x.into()).collect(),
        })
    }

    pub async fn api_key_update(
        &self,
        client: &Client,
        req: RequestApiKeyUpdate,
    ) -> Result<ResponseApiKey> {
        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("api_key_update.sql")).await?;

        let row = conn
            .query_one(
                &statement,
                &[&client.client_id, &req.id, &req.name, &req.enable],
            )
            .await?;

        Ok(row.into())
    }

    pub async fn api_key_delete(&self, client: &Client, req: RequestApiKeyDelete) -> Result<()> {
        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("api_key_delete.sql")).await?;
        conn.query(&statement, &[&client.client_id, &req.id])
            .await?;

        Ok(())
    }

    pub async fn api_key_verify(
        &self,
        client: &Client,
        req: RequestApiKeyVerify,
    ) -> Result<ResponseApiKey> {
        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("api_key_verify.sql")).await?;

        let (key_id, key_secret) = self.key_secret_extract(&req.key)?;

        let rows = conn
            .query(&statement, &[&client.client_id, &key_id, &key_secret])
            .await?;

        if !rows.is_empty() {
            let row = &rows[0];
            Ok(row.into())
        } else {
            Err(Error::from("api key verify failed"))
        }
    }

    pub async fn audit_insert(&self, audit: Audit) -> Result<ResponseAudit> {
        let conn = self.pool.get().await?;

        info!("{}", audit);

        let statement = conn.prepare(include_str!("audit_insert.sql")).await?;
        let row = conn
            .query_one(
                &statement,
                &[
                    &audit.client_id,
                    &audit.user_id,
                    &audit.token_id,
                    &audit.api_key_id,
                    &audit.audit_type,
                    &audit.subject,
                    &audit.data,
                    &audit.status_code,
                ],
            )
            .await?;

        Ok(row.into())
    }

    pub async fn audit_read_id(&self, id: i64) -> Result<Option<ResponseAudit>> {
        let conn = self.pool.get().await?;
        PostgresQuery::audit_read_id(&conn, id).await
    }

    pub async fn audit_retention(&self, days: i32) -> Result<u64> {
        let conn = self.pool.get().await?;
        PostgresQuery::audit_retention(&conn, days).await
    }

    pub async fn audit_read(
        &self,
        client: &Client,
        req: RequestAuditRead,
    ) -> Result<ResponseAuditMany> {
        let conn = self.pool.get().await?;

        let statement = conn.prepare(include_str!("audit_read.sql")).await?;
        let rows = conn
            .query(
                &statement,
                &[
                    &client.client_id,
                    &req.seek.id,
                    &req.seek.limit,
                    &req.id,
                    &req.user_id,
                    &req.audit_type,
                    &req.subject,
                ],
            )
            .await?;

        Ok(ResponseAuditMany {
            data: rows.into_iter().map(|x| x.into()).collect(),
        })
    }

    pub fn key_secret_extract(&self, client_secret: &str) -> Result<(Uuid, String)> {
        let parts: Vec<&str> = client_secret.split('.').collect();
        if parts.len() == 2 {
            let id = parts.get(0).unwrap();
            let secret = parts.get(1).unwrap();

            match Uuid::parse_str(id) {
                Ok(id) => Ok((id, secret.to_string())),
                Err(_e) => Err(Error::from("client_secret is invalid")),
            }
        } else {
            Err(Error::from("client_secret is invalid"))
        }
    }
}

impl PostgresQuery {
    async fn readiness(conn: &deadpool_postgres::Client) -> Result<()> {
        let st = conn.prepare("SELECT 1").await?;
        conn.query_one(&st, &[]).await?;
        Ok(())
    }

    async fn secret_generate(conn: &deadpool_postgres::Client, count: i32) -> Result<String> {
        let st = conn.prepare("SELECT sso._secret_generate($1)").await?;
        let row = conn.query_one(&st, &[&count]).await?;
        Ok(row.get(0))
    }

    async fn secret_hash(
        conn: &deadpool_postgres::Client,
        secret: &str,
        value: &str,
    ) -> Result<String> {
        let st = conn.prepare("SELECT sso._secret_hash($1, $2)").await?;
        let row = conn.query_one(&st, &[&secret, &value]).await?;
        Ok(row.get(0))
    }

    async fn user_upsert_static(
        conn: &deadpool_postgres::Client,
        id: &Uuid,
        user: &ConfigOauth2User,
    ) -> Result<u64> {
        let st = conn.prepare(include_str!("user/upsert_static.sql")).await?;
        let rows = conn
            .execute(
                &st,
                &[
                    id,
                    &user.name,
                    &user.email,
                    &user.locale,
                    &user.timezone,
                    &user.enable,
                ],
            )
            .await?;
        Ok(rows)
    }

    async fn user_password_upsert(
        conn: &deadpool_postgres::Client,
        id: &Uuid,
        password_hash: &str,
        allow_reset: bool,
        require_update: bool,
        static_: bool,
    ) -> Result<ResponseUserPassword> {
        let st = conn
            .prepare(include_str!("user/password_upsert.sql"))
            .await?;
        let row = conn
            .query_one(
                &st,
                &[id, &password_hash, &allow_reset, &require_update, &static_],
            )
            .await?;
        Ok(row.into())
    }

    async fn user_delete_static(
        conn: &deadpool_postgres::Client,
        exclude_id: &Vec<Uuid>,
    ) -> Result<u64> {
        let st = conn.prepare(include_str!("user/delete_static.sql")).await?;
        let rows = conn.execute(&st, &[exclude_id]).await?;
        Ok(rows)
    }

    async fn access_upsert(
        conn: &deadpool_postgres::Client,
        client_id: &Uuid,
        user_id: &Uuid,
        enable: bool,
        scope: &oauth2::Scope,
        static_: bool,
    ) -> Result<ResponseAccess> {
        let st = conn.prepare(include_str!("access/upsert.sql")).await?;
        let row = conn
            .query_one(
                &st,
                &[client_id, user_id, &enable, &scope.to_string(), &static_],
            )
            .await?;
        Ok(row.into())
    }

    async fn audit_read_id(
        conn: &deadpool_postgres::Client,
        id: i64,
    ) -> Result<Option<ResponseAudit>> {
        let st = conn.prepare(include_str!("audit/read_id.sql")).await?;
        let mut rows = conn.query(&st, &[&id]).await?;
        if rows.is_empty() {
            Ok(None)
        } else {
            Ok(Some(rows.remove(0).into()))
        }
    }

    async fn audit_retention(conn: &deadpool_postgres::Client, days: i32) -> Result<u64> {
        let st = conn.prepare(include_str!("audit/retention.sql")).await?;
        let rows = conn.execute(&st, &[&days]).await?;
        Ok(rows)
    }
}

pub(crate) struct PostgresUserPasswordCheck {
    pub id: Uuid,
    pub check: bool,
    pub enable: bool,
    pub require_update: bool,
}

impl From<Row> for PostgresUserPasswordCheck {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            check: row.get("check"),
            enable: row.get("enable"),
            require_update: row.get("require_update"),
        }
    }
}

struct AccessRead {
    scope: String,
    enable: bool,
}

impl From<Row> for AccessRead {
    fn from(row: Row) -> Self {
        Self {
            scope: row.get("scope"),
            enable: row.get("enable"),
        }
    }
}

impl From<Row> for ResponseUserPassword {
    fn from(row: Row) -> Self {
        Self {
            user_id: row.get("user_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            allow_reset: row.get("allow_reset"),
            require_update: row.get("require_update"),
            static_: row.get("static"),
        }
    }
}

impl From<Row> for ResponseAudit {
    fn from(row: Row) -> Self {
        Self {
            created_at: row.get("created_at"),
            id: row.get("id"),
            client_id: row.get("client_id"),
            user_id: row.get("user_id"),
            token_id: row.get("token_id"),
            api_key_id: row.get("api_key_id"),
            audit_type: row.get("type"),
            subject: row.get("subject"),
            data: row.get("data"),
            status_code: row.get("status_code"),
        }
    }
}

impl From<Row> for ResponseUser {
    fn from(row: Row) -> Self {
        Self {
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            locale: row.get("locale"),
            timezone: row.get("timezone"),
            enable: row.get("enable"),
            static_: row.get("static"),
            password: ResponseUserPassword::try_from(&row),
            oauth2_provider: Vec::new(),
            oauth2_provider_count: row.get("oauth2_provider_count"),
            access: ResponseAccess::try_from(&row),
        }
    }
}

impl ResponseUserPassword {
    fn try_from(row: &Row) -> Option<Self> {
        match row.try_get("password_user_id") {
            Ok(user_id) => Some(ResponseUserPassword {
                created_at: row.get("password_created_at"),
                updated_at: row.get("password_updated_at"),
                user_id,
                allow_reset: row.get("password_allow_reset"),
                require_update: row.get("password_require_update"),
                static_: row.get("password_static"),
            }),
            Err(_e) => None,
        }
    }
}

impl From<Row> for ResponseAccess {
    fn from(row: Row) -> Self {
        Self {
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            client_id: row.get("client_id"),
            user_id: row.get("user_id"),
            enable: row.get("enable"),
            scope: row.get("scope"),
            static_: row.get("static"),
        }
    }
}

impl ResponseAccess {
    fn try_from(row: &Row) -> Option<Self> {
        match row.try_get("access_user_id") {
            Ok(user_id) => Some(ResponseAccess {
                created_at: row.get("access_created_at"),
                updated_at: row.get("access_updated_at"),
                client_id: row.get("access_client_id"),
                user_id,
                enable: row.get("access_enable"),
                scope: row.get("access_scope"),
                static_: row.get("access_static"),
            }),
            Err(_e) => None,
        }
    }
}

impl From<Row> for ResponseApiKey {
    fn from(row: Row) -> Self {
        (&row).into()
    }
}

impl From<&Row> for ResponseApiKey {
    fn from(row: &Row) -> Self {
        Self {
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            id: row.get("id"),
            client_id: row.get("client_id"),
            user_id: row.get("user_id"),
            name: row.get("name"),
            enable: row.get("enable"),
            scope: row.get("scope"),
            value: None,
        }
    }
}

/// Audit
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Audit {
    pub client_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub token_id: Option<Uuid>,
    pub api_key_id: Option<Uuid>,
    pub audit_type: String,
    pub subject: Option<String>,
    pub data: Value,
    pub status_code: Option<i16>,
}

impl std::fmt::Display for Audit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl Audit {
    pub fn from_http_request(audit_type: &str, req: &actix_web::HttpRequest) -> Self {
        let mut audit = Self {
            client_id: None,
            user_id: None,
            token_id: None,
            api_key_id: None,
            audit_type: audit_type.to_string(),
            subject: None,
            data: json!({}),
            status_code: None,
        };
        audit.set_data_http_request(req);
        audit
    }

    pub fn set_client(&mut self, client: &Client) {
        self.client_id = Some(client.client_id);
    }

    pub fn set_user_id(&mut self, user_id: Uuid) {
        self.user_id = Some(user_id);
    }

    pub fn set_status_ok(&mut self) {
        self.status_code = Some(200);
    }

    pub fn set_data(&mut self, key: &str, data: Value) {
        let obj = self.data.as_object_mut().unwrap();
        obj.insert(key.to_string(), data);
    }

    pub fn set_data_http_request(&mut self, req: &actix_web::HttpRequest) {
        let info = req.connection_info();
        let headers = req.headers();
        let user_agent = headers.get("user-agent").map(|x| x.to_str().unwrap());

        self.set_data(
            "http_request",
            json!({
                "scheme": info.scheme(),
                "host": info.host(),
                "remote": info.remote(),
                "user_agent": user_agent,
            }),
        );
    }

    pub fn set_data_err(&mut self, err: &oauth2::ErrorResponse) {
        self.set_data(
            "error",
            json!({
                "code": err.error().as_str(),
                "description": err.error_description(),
            }),
        );
    }

    pub fn set_status_err(&mut self, err: &oauth2::ErrorResponse) {
        self.status_code = Some(match err.error() {
            oauth2::ErrorCode::InvalidRequest => 400,
            oauth2::ErrorCode::UnauthorizedClient => 401,
            oauth2::ErrorCode::AccessDenied => 403,
            oauth2::ErrorCode::UnsupportedResponseType => 400,
            oauth2::ErrorCode::InvalidScope => 400,
            oauth2::ErrorCode::ServerError => 500,
            oauth2::ErrorCode::TemporarilyUnavailable => 503,
        });
    }

    pub fn from_create_request(client: &Client, req: RequestAuditCreate) -> Self {
        Self {
            client_id: Some(client.client_id),
            user_id: req.user_id,
            token_id: req.token_id,
            api_key_id: req.api_key_id,
            audit_type: req.audit_type,
            subject: req.subject,
            data: req.data,
            status_code: req.status_code,
        }
    }
}

/// Code Target Postgres Type
#[derive(Debug, Clone, ToSql, FromSql)]
#[postgres(name = "sso_code_target")]
pub(crate) enum PostgresCodeTarget {
    Auth,
    PasswordReset,
    Register,
    Delete,
}

/// OAuth2 Provider Postgres Type
#[derive(Debug, Clone, ToSql, FromSql)]
#[postgres(name = "sso_oauth2_provider")]
pub(crate) enum PostgresOauth2Provider {
    Sso,
    Microsoft,
}

impl PostgresOauth2Provider {
    pub fn from_str(x: &str) -> oauth2::Result<Self> {
        match x {
            "sso" => Ok(Self::Sso),
            "microsoft" => Ok(Self::Microsoft),
            _ => Err(oauth2::ErrorResponse::invalid_request(
                "oauth2_provider is invalid",
            )),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Sso => "sso",
            Self::Microsoft => "microsoft",
        }
    }
}

/// OAuth2 Target Postgres Type
#[derive(Debug, Clone, ToSql, FromSql)]
#[postgres(name = "sso_oauth2_target")]
pub(crate) enum PostgresOauth2Target {
    Auth,
    Register,
}

/// Code Postgres Type
#[derive(Debug, Clone)]
pub(crate) struct PostgresCode {
    pub client_id: Uuid,
    pub user_id: Option<Uuid>,
    pub state: String,
    pub scope: oauth2::Scope,
    pub email: String,
}

/// OAuth2 Code Postgres Type
#[derive(Debug, Clone)]
pub(crate) struct PostgresOauth2Code {
    pub provider: PostgresOauth2Provider,
    pub target: PostgresOauth2Target,
    pub pkce: String,
    pub redirect_uri: Option<Url>,
    pub state: String,
    pub scope: String,
    pub email: String,
}

/// Token Postgres Type
#[derive(Debug, Clone)]
pub(crate) struct PostgresToken {
    pub access_token: String,
    pub refresh_token: String,
    pub scope: oauth2::Scope,
}
