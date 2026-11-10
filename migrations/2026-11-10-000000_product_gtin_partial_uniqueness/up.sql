-- Replace the full unique constraint on gtin with a partial unique index that
-- only enforces uniqueness for non-empty values, allowing multiple products to
-- have an empty gtin when it is not applicable.
ALTER TABLE products DROP CONSTRAINT IF EXISTS products_gtin_key;
CREATE UNIQUE INDEX products_gtin_nonempty_key ON products (gtin) WHERE gtin <> '';
