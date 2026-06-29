-- This file should undo anything in `up.sql`
DROP INDEX orders_payment_reference_idx;

ALTER TABLE "orders"
DROP COLUMN payment_reference;
