-- Your SQL goes here
CREATE TYPE dbpaymentstatus AS ENUM (
    'Pending', 'Paid', 'Failed', 'Refunded'
);

ALTER TABLE "orders"
ADD COLUMN payment_status dbpaymentstatus NOT NULL DEFAULT 'Pending';
