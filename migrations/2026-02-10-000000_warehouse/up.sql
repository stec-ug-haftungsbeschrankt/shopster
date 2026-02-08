-- Your SQL goes here
CREATE TABLE "warehouse" (
    id BIGSERIAL PRIMARY KEY,
    product_id BIGINT NOT NULL UNIQUE,
    in_stock BIGINT NOT NULL DEFAULT 0,
    reserved BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP
);
