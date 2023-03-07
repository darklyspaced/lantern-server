// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Int4,
        email -> Varchar,
        firefly_secret -> Varchar,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        firefly_secret -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    posts,
    users,
);
