// @generated automatically by Diesel CLI.

diesel::table! {
    keys (id) {
        id -> Int4,
        address -> Varchar,
        local_share -> Text,
    }
}
