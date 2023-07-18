// @generated automatically by Diesel CLI.

diesel::table! {
    paths (id) {
        id -> Text,
        path -> Text,
        prefix -> Text,
    }
}

diesel::table! {
    projects (id) {
        id -> Text,
        name -> Text,
        path_id -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    paths,
    projects,
);
