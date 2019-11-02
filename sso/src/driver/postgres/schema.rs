table! {
    sso_audit (created_at, id) {
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        id -> Uuid,
        user_agent -> Varchar,
        remote -> Varchar,
        forwarded -> Nullable<Varchar>,
        status_code -> Nullable<Int2>,
        #[sql_name = "type"]
        type_ -> Varchar,
        subject -> Nullable<Varchar>,
        data -> Jsonb,
        key_id -> Nullable<Uuid>,
        service_id -> Nullable<Uuid>,
        user_id -> Nullable<Uuid>,
        user_key_id -> Nullable<Uuid>,
    }
}

table! {
    sso_csrf (key) {
        created_at -> Timestamptz,
        key -> Varchar,
        value -> Varchar,
        ttl -> Timestamptz,
        service_id -> Uuid,
    }
}

table! {
    sso_key (id) {
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        id -> Uuid,
        is_enabled -> Bool,
        is_revoked -> Bool,
        #[sql_name = "type"]
        type_ -> Varchar,
        name -> Varchar,
        value -> Varchar,
        service_id -> Nullable<Uuid>,
        user_id -> Nullable<Uuid>,
    }
}

table! {
    sso_service (id) {
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        id -> Uuid,
        is_enabled -> Bool,
        name -> Varchar,
        url -> Varchar,
        user_allow_register -> Bool,
        user_email_text -> Varchar,
        provider_local_url -> Nullable<Varchar>,
        provider_github_oauth2_url -> Nullable<Varchar>,
        provider_microsoft_oauth2_url -> Nullable<Varchar>,
    }
}

table! {
    sso_user (id) {
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        id -> Uuid,
        is_enabled -> Bool,
        name -> Varchar,
        email -> Varchar,
        locale -> Varchar,
        timezone -> Varchar,
        password_allow_reset -> Bool,
        password_require_update -> Bool,
        password_hash -> Nullable<Varchar>,
    }
}

joinable!(sso_audit -> sso_service (service_id));
joinable!(sso_audit -> sso_user (user_id));
joinable!(sso_csrf -> sso_service (service_id));
joinable!(sso_key -> sso_service (service_id));
joinable!(sso_key -> sso_user (user_id));

allow_tables_to_appear_in_same_query!(sso_audit, sso_csrf, sso_key, sso_service, sso_user,);
