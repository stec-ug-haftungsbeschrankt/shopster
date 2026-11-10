-- Applied coupon code and resulting discount on an order, so reports and
-- refunds can reconcile with what was actually charged.
ALTER TABLE orders
    ADD COLUMN coupon_code    TEXT,
    ADD COLUMN discount_amount BIGINT,
    ADD COLUMN charged_total   BIGINT;
