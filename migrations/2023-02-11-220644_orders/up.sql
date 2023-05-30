-- Your SQL goes here
CREATE TYPE OrderStatus AS ENUM (
    'New', 'InProgress', 'ReadyToShip', 'Shipping', 'Done'
);

CREATE TABLE "orders" (
    id BIGSERIAL PRIMARY KEY,
    status OrderStatus NOT NULL,
    delivery_address TEXT NOT NULL,
    billing_address TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP
);
