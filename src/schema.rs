// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        firefly_secret -> Varchar,
    }
}
