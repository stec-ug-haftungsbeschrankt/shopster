-- Per-product low-stock threshold. When NULL the shop-wide default is used.
ALTER TABLE warehouse ADD COLUMN low_stock_threshold BIGINT;
