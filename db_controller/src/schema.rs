// @generated automatically by Diesel CLI.

diesel::table! {
    active_sessions (id) {
        id -> Integer,
        user -> Text,
        service_id -> Integer,
        docker_id -> Nullable<Text>,
        container_ip -> Nullable<Text>,
        port -> Nullable<Integer>,
        unix_created_at -> BigInt,
    }
}

diesel::table! {
    invite_key (inv_key) {
        inv_key -> Text,
        unix_created_at -> BigInt,
    }
}

diesel::table! {
    services (id) {
        id -> Integer,
        name -> Text,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    users (username) {
        username -> Text,
        password -> Nullable<Text>,
        role -> Text,
    }
}

diesel::joinable!(active_sessions -> services (service_id));
diesel::joinable!(active_sessions -> users (user));

diesel::allow_tables_to_appear_in_same_query!(
    active_sessions,
    invite_key,
    services,
    users,
);
