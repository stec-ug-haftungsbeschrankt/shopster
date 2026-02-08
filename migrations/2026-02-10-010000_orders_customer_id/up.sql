-- Your SQL goes here
ALTER TABLE "orders"
ADD COLUMN customer_id UUID REFERENCES customers(id);
