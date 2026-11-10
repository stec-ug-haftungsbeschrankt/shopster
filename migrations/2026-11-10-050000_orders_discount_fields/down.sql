ALTER TABLE orders
    DROP COLUMN IF EXISTS coupon_code,
    DROP COLUMN IF EXISTS discount_amount,
    DROP COLUMN IF EXISTS charged_total;
