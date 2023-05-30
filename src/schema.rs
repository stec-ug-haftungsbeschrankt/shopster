// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "orderstatus"))]
    pub struct Orderstatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Orderstatus;

    orders (id) {
        id -> Int8,
        status -> Orderstatus,
        delivery_address -> Text,
        billing_address -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    products (id) {
        id -> Int8,
        article_number -> Text,
        gtin -> Text,
        title -> Text,
        short_description -> Text,
        description -> Text,
        tags -> Text,
        title_image -> Text,
        additional_images -> Text,
        price -> Int8,
        currency -> Text,
        weight -> Int4,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    settings (id) {
        id -> Int4,
        title -> Text,
        datatype -> Text,
        value -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        email -> Text,
        email_verified -> Bool,
        algorithm -> Text,
        password -> Text,
        full_name -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    orders,
    products,
    settings,
    users,
);
