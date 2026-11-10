-- Marks settings whose value is a secret (API keys, webhook signing secrets).
-- Secret values are encrypted by the application before they are written and are
-- excluded from Settings::get_all.
ALTER TABLE settings ADD COLUMN is_secret BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE settings SET is_secret = TRUE
WHERE title IN ('stripe_secret_api_key', 'stripe_webhook_secret', 'sendgrid_api_key');
