-- This file should undo anything in `up.sql`

DELETE FROM settings WHERE title = 'stripe_webhook_secret';
