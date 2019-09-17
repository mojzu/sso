table! {
    auth_audit (audit_id) {
        created_at -> Timestamptz,
        audit_id -> Uuid,
        audit_user_agent -> Varchar,
        audit_remote -> Varchar,
        audit_forwarded -> Nullable<Varchar>,
        audit_type -> Varchar,
        audit_data -> Jsonb,
        key_id -> Nullable<Uuid>,
        service_id -> Nullable<Uuid>,
        user_id -> Nullable<Uuid>,
        user_key_id -> Nullable<Uuid>,
    }
}

table! {
    auth_csrf (csrf_key) {
        created_at -> Timestamptz,
        csrf_key -> Varchar,
        csrf_value -> Varchar,
        csrf_ttl -> Timestamptz,
        service_id -> Uuid,
    }
}

table! {
    auth_key (key_id) {
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        key_id -> Uuid,
        key_is_enabled -> Bool,
        key_is_revoked -> Bool,
        key_name -> Varchar,
        key_value -> Varchar,
        service_id -> Nullable<Uuid>,
        user_id -> Nullable<Uuid>,
    }
}

table! {
    auth_service (service_id) {
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        service_id -> Uuid,
        service_is_enabled -> Bool,
        service_name -> Varchar,
        service_url -> Varchar,
    }
}

table! {
    auth_user (user_id) {
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        user_id -> Uuid,
        user_is_enabled -> Bool,
        user_name -> Varchar,
        user_email -> Varchar,
        user_password_hash -> Nullable<Varchar>,
    }
}

joinable!(auth_audit -> auth_service (service_id));
joinable!(auth_audit -> auth_user (user_id));
joinable!(auth_csrf -> auth_service (service_id));
joinable!(auth_key -> auth_service (service_id));
joinable!(auth_key -> auth_user (user_id));

allow_tables_to_appear_in_same_query!(auth_audit, auth_csrf, auth_key, auth_service, auth_user,);
