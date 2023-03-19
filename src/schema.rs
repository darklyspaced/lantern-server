// @generated automatically by Diesel CLI.

diesel::table! {
    tasks (id) {
        id -> Int4,
        user_email -> Varchar,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        firefly_secret -> Varchar,
        device_id -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(tasks, users,);
