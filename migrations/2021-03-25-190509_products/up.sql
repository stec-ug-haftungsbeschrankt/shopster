-- Your SQL goes here


CREATE TABLE "products" (
    id BIGSERIAL PRIMARY KEY,
    article_number TEXT UNIQUE NOT NULL,
    gtin TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    short_description TEXT NOT NULL,
    description TEXT NOT NULL,
    tags TEXT NOT NULL,
    title_image TEXT NOT NULL,
    additional_images TEXT NOT NULL,
    price BIGINT NOT NULL,
    currency TEXT NOT NULL,
    weight INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP
);
