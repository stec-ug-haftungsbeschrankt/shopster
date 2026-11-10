DROP INDEX IF EXISTS products_gtin_nonempty_key;
ALTER TABLE products ADD CONSTRAINT products_gtin_key UNIQUE (gtin);
