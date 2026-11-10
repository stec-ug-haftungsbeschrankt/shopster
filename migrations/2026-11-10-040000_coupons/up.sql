CREATE TABLE coupons (
    id              BIGSERIAL PRIMARY KEY,
    code            TEXT NOT NULL,
    -- Mutually exclusive: one of percent_off or amount_off must be set.
    percent_off     INTEGER,            -- 0-100
    amount_off      BIGINT,             -- fixed amount in minor currency units
    currency        TEXT,               -- required when amount_off is set
    valid_from      TIMESTAMP,
    valid_until     TIMESTAMP,
    max_uses        INTEGER,            -- NULL = unlimited
    use_count       INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX coupons_code_key ON coupons (code);
