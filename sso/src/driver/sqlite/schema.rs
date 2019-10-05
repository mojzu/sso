table! {
    sso_audit (audit_id) {
        created_at -> Text,
        audit_id -> Text,
        audit_user_agent -> Text,
        audit_remote -> Text,
        audit_forwarded -> Nullable<Text>,
        audit_type -> Text,
        audit_data -> Binary,
        key_id -> Nullable<Text>,
        service_id -> Nullable<Text>,
        user_id -> Nullable<Text>,
        user_key_id -> Nullable<Text>,
    }
}

table! {
    sso_csrf (csrf_key) {
        created_at -> Text,
        csrf_key -> Text,
        csrf_value -> Text,
        csrf_ttl -> Text,
        service_id -> Text,
    }
}

table! {
    sso_key (key_id) {
        created_at -> Text,
        updated_at -> Text,
        key_id -> Text,
        key_is_enabled -> Bool,
        key_is_revoked -> Bool,
        key_name -> Text,
        key_value -> Text,
        service_id -> Nullable<Text>,
        user_id -> Nullable<Text>,
    }
}

table! {
    sso_service (service_id) {
        created_at -> Text,
        updated_at -> Text,
        service_id -> Text,
        service_is_enabled -> Bool,
        service_name -> Text,
        service_url -> Text,
    }
}

table! {
    sso_user (user_id) {
        created_at -> Text,
        updated_at -> Text,
        user_id -> Text,
        user_is_enabled -> Bool,
        user_name -> Text,
        user_email -> Text,
        user_password_hash -> Nullable<Text>,
    }
}

joinable!(sso_audit -> sso_service (service_id));
joinable!(sso_audit -> sso_user (user_id));
joinable!(sso_csrf -> sso_service (service_id));
joinable!(sso_key -> sso_service (service_id));
joinable!(sso_key -> sso_user (user_id));

allow_tables_to_appear_in_same_query!(sso_audit, sso_csrf, sso_key, sso_service, sso_user,);
