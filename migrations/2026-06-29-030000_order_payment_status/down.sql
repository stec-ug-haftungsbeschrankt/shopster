-- This file should undo anything in `up.sql`
ALTER TABLE "orders"
DROP COLUMN payment_status;

DROP TYPE dbpaymentstatus;
