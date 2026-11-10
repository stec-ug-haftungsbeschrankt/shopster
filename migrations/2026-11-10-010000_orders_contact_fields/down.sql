ALTER TABLE orders
    DROP COLUMN IF EXISTS contact_email,
    DROP COLUMN IF EXISTS contact_name;
