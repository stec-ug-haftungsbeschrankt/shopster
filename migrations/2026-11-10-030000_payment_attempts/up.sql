CREATE TABLE payment_attempts (
    id              BIGSERIAL PRIMARY KEY,
    payment_intent_id TEXT NOT NULL,
    reason          TEXT NOT NULL,
    basket_id       UUID,
    amount          BIGINT NOT NULL,
    currency        TEXT NOT NULL,
    created_at      TIMESTAMP NOT NULL DEFAULT current_timestamp
);

-- Unique index on payment_intent_id for idempotent re-delivery of Stripe webhooks.
CREATE UNIQUE INDEX payment_attempts_payment_intent_id_key ON payment_attempts (payment_intent_id);
