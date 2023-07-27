-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE "baskets" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),   
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP
);

CREATE TABLE "basketproducts" (
  id BIGSERIAL PRIMARY KEY,
  product_id BIGSERIAL,
  quantity BIGINT NOT NULL,
  basket_id UUID references baskets(id)
)