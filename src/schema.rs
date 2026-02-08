// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "dborderstatus"))]
    pub struct DbOrderStatus;
}

diesel::table! {
    basketproducts (id) {
        id -> Int8,
        product_id -> Int8,
        quantity -> Int8,
        basket_id -> Uuid,
    }
}

diesel::table! {
    baskets (id) {
        id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    customers (id) {
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

diesel::table! {
    order_items (id) {
        id -> Int8,
        order_id -> Int8,
        product_id -> Int8,
        quantity -> Int8,
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
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::DbOrderStatus;

    orders (id) {
        id -> Int8,
        customer_id -> Nullable<Uuid>,
        status -> DbOrderStatus,
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
    warehouse (id) {
        id -> Int8,
        product_id -> Int8,
        in_stock -> Int8,
        reserved -> Int8,
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

diesel::joinable!(basketproducts -> baskets (basket_id));
diesel::joinable!(order_items -> orders (order_id));
diesel::joinable!(orders -> customers (customer_id));

diesel::allow_tables_to_appear_in_same_query!(
    basketproducts,
    baskets,
    customers,
    order_items,
    orders,
    products,
    settings,
    warehouse,
    users,
);
