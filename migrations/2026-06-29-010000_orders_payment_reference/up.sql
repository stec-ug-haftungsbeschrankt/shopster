-- Your SQL goes here
ALTER TABLE "orders"
ADD COLUMN payment_reference TEXT;

CREATE UNIQUE INDEX orders_payment_reference_idx ON orders (payment_reference) WHERE payment_reference IS NOT NULL;
