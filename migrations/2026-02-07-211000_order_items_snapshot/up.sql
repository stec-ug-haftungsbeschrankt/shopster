CREATE TABLE order_items (
    id BIGSERIAL PRIMARY KEY,
    order_id BIGINT NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id BIGINT NOT NULL,
    quantity BIGINT NOT NULL,
    article_number TEXT NOT NULL,
    gtin TEXT NOT NULL,
    title TEXT NOT NULL,
    short_description TEXT NOT NULL,
    description TEXT NOT NULL,
    tags TEXT NOT NULL,
    title_image TEXT NOT NULL,
    additional_images TEXT NOT NULL,
    price BIGINT NOT NULL,
    currency TEXT NOT NULL,
    weight INT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
