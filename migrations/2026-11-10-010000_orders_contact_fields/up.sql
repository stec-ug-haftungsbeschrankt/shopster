-- Add optional contact fields to orders for guest (storefront) orders where
-- customer_id is NULL, so notifications and admin views have a name and email.
ALTER TABLE orders
    ADD COLUMN contact_email TEXT,
    ADD COLUMN contact_name  TEXT;
