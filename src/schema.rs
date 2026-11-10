// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "dborderstatus"))]
    pub struct DbOrderStatus;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "dbpaymentstatus"))]
    pub struct DbPaymentStatus;
}

diesel::table! {
    coupons (id) {
        id -> Int8,
        code -> Text,
        percent_off -> Nullable<Int4>,
        amount_off -> Nullable<Int8>,
        currency -> Nullable<Text>,
        valid_from -> Nullable<Timestamp>,
        valid_until -> Nullable<Timestamp>,
        max_uses -> Nullable<Int4>,
        use_count -> Int4,
        created_at -> Timestamp,
    }
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
    use super::sql_types::DbPaymentStatus;

    orders (id) {
        id -> Int8,
        customer_id -> Nullable<Uuid>,
        status -> DbOrderStatus,
        delivery_address -> Text,
        billing_address -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        payment_reference -> Nullable<Text>,
        payment_status -> DbPaymentStatus,
        coupon_code -> Nullable<Text>,
        discount_amount -> Nullable<Int8>,
        charged_total -> Nullable<Int8>,
        contact_email -> Nullable<Text>,
        contact_name -> Nullable<Text>,
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
        low_stock_threshold -> Nullable<Int8>,
    }
}

diesel::table! {
    payment_attempts (id) {
        id -> Int8,
        payment_intent_id -> Text,
        reason -> Text,
        basket_id -> Nullable<Uuid>,
        amount -> Int8,
        currency -> Text,
        created_at -> Timestamp,
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
    coupons,
    customers,
    order_items,
    orders,
    payment_attempts,
    products,
    settings,
    warehouse,
    users,
);
