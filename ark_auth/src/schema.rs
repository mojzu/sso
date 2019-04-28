table! {
    auth_csrf (csrf_key) {
        created_at -> Timestamptz,
        csrf_key -> Varchar,
        csrf_value -> Varchar,
        service_id -> Int8,
    }
}

table! {
    auth_key (key_id) {
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        key_id -> Int8,
        key_name -> Varchar,
        key_value -> Varchar,
        service_id -> Int8,
        user_id -> Nullable<Int8>,
    }
}

table! {
    auth_service (service_id) {
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        service_id -> Int8,
        service_name -> Varchar,
        service_url -> Varchar,
    }
}

table! {
    auth_user (user_id) {
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        user_id -> Int8,
        user_name -> Varchar,
        user_email -> Varchar,
        user_password -> Nullable<Varchar>,
        user_password_revision -> Nullable<Int8>,
    }
}

joinable!(auth_csrf -> auth_service (service_id));
joinable!(auth_key -> auth_service (service_id));
joinable!(auth_key -> auth_user (user_id));

allow_tables_to_appear_in_same_query!(auth_csrf, auth_key, auth_service, auth_user,);
