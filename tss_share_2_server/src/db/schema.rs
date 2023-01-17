// @generated automatically by Diesel CLI.

diesel::table! {
    keys (id) {
        id -> Int4,
        address -> Nullable<Varchar>,
        local_share -> Nullable<Text>,
    }
}

diesel::table! {
    share2_keys (id) {
        id -> Int4,
        address -> Varchar,
        local_share -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    keys,
    share2_keys,
);
