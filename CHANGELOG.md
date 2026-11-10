# Changelog

All notable changes to this project will be documented in this file.

## [0.7.0]

### Changed (breaking)
- Removed the process-wide `DATABASE_SELECTOR` singleton. `Shopster` now owns its `DatabaseSelector` (behind an `Arc`) and hands every domain handler a `TenantDb` handle, so multiple `Shopster` instances (e.g. different Tenet backends, isolated tests) can coexist in one process. A second `Shopster::new` no longer silently reuses the first instance's tenants. ([#39](https://github.com/stec-ug-haftungsbeschrankt/shopster/issues/39))
- `Baskets::new`, `Orders::new`, ... now take a `TenantDb` instead of a tenant `Uuid`; prefer the `Shopster::baskets(tenant_id)` etc. accessors. `ShopsterError::NotInitialized` is no longer produced.

### Added
- `Shopster::from_shared(Arc<DatabaseSelector>)` and the public `TenantDb` type.

### Dependencies
- `cargo update` to the latest semver-compatible versions; `aes-gcm` 0.10 -> 0.11.

## [0.6.0]

### Security
- `.env` is no longer tracked and is git-ignored; `.env.example` documents the variables. ([#33](https://github.com/stec-ug-haftungsbeschrankt/shopster/issues/33))
- Tenant connection strings are no longer logged with their password (`redact_connection_string`). ([#34](https://github.com/stec-ug-haftungsbeschrankt/shopster/issues/34))
- `DatabaseHelper` parses connection strings properly, validates database names (`[A-Za-z0-9_-]`, max 63) before using them in `CREATE`/`DROP DATABASE`, returns errors instead of panicking, and `is_database_exists` now distinguishes a missing database from an unreachable server or bad credentials. ([#35](https://github.com/stec-ug-haftungsbeschrankt/shopster/issues/35))
- Passwords are hashed with **Argon2id** (19 MiB, t=2, p=1) instead of the legacy Argon2i defaults. Old hashes keep verifying and are rehashed on successful login. Unknown e-mail and wrong password now both return `AuthenticationError` (previously the unknown e-mail returned a `DatabaseError`) and take comparable time. ([#37](https://github.com/stec-ug-haftungsbeschrankt/shopster/issues/37))
- Secret settings (`stripe_secret_api_key`, `stripe_webhook_secret`, `sendgrid_api_key`) are encrypted at rest (AES-256-GCM, key supplied via `DatabaseSelector::with_secret_key`). New `Settings::get_secret`, `set_secret`, `encrypt_legacy_secrets`. ([#38](https://github.com/stec-ug-haftungsbeschrankt/shopster/issues/38))

### Changed (breaking)
- `Settings::get_all` no longer returns secret settings; `Setting` gained `is_secret` and `value` is empty for secrets; `Settings::update_by_id` refuses secret settings (use `set_secret`).
- A failed `CREATE DATABASE` or migration now makes `DatabaseSelector::get_storage_for_tenant` return `ShopsterError::DatabaseMigrationError` / the underlying error and is retried on the next call, instead of logging a warning and caching a pool for a half-migrated schema. ([#36](https://github.com/stec-ug-haftungsbeschrankt/shopster/issues/36))
- Using a handler before `Shopster::new` returns the new `ShopsterError::NotInitialized` instead of panicking. (Partially addresses [#39](https://github.com/stec-ug-haftungsbeschrankt/shopster/issues/39); the global `DatabaseSelector` singleton is still in place.)
- `DatabaseHelper::create_database`/`is_database_exists`/`drop_database` return `ShopsterError` results (internal module).
- `simple_logger` moved to dev-dependencies; applications choose their own logger. ([#40](https://github.com/stec-ug-haftungsbeschrankt/shopster/issues/40))

### Migrations
- `2026-11-10-060000_settings_secrets` (adds `settings.is_secret`, flags the three secret settings)

## [0.5.0]

### Added
- `OrderStatus::Cancelled` is a new terminal order status representing an order that will never be fulfilled (customer cancellation, stock unavailable, fraud check failure, permanent payment failure, etc). `Orders::is_valid_transition` allows cancellation from any non-terminal status (`New`, `InProgress`, `ReadyToShip`, `Shipping`) into `Cancelled`, and disallows any transition out of `Cancelled` or from `Done`. Cancelling a reserving order releases its warehouse reservation through the same mechanism used for other status transitions. ([#10](https://github.com/stec-ug-haftungsbeschrankt/shopster/issues/10))
- `PaymentStatus` enum (`Pending`, `Paid`, `Failed`, `Refunded`) and matching `DbPaymentStatus`, tracked independently of `OrderStatus` since payment and fulfillment are orthogonal (e.g. an order can be `Cancelled` after being `Paid` and still need a refund). `Order` and `DbOrder` gained a `payment_status` field, defaulting to `Pending`. ([#11](https://github.com/stec-ug-haftungsbeschrankt/shopster/issues/11))
- `Orders::update_payment_status` updates an order's payment status directly, bypassing fulfillment transition validation — intended for use by payment webhooks (e.g. Stripe) that need to record a payment outcome without driving the fulfillment lifecycle.

### Migrations
- `2026-06-29-020000_order_cancelled_status` (adds the `Cancelled` value to the `dborderstatus` enum; runs outside a transaction, as required by `ALTER TYPE ... ADD VALUE`)
- `2026-06-29-030000_order_payment_status` (adds the `dbpaymentstatus` enum and the `orders.payment_status` column, default `Pending`)

## [0.4.0]

### Added
- `orders::Order` and `DbOrder` now have a `payment_reference` field (nullable, unique) for storing an external payment reference such as a Stripe Checkout Session ID or PaymentIntent ID. ([#7](https://github.com/stec-ug-haftungsbeschrankt/shopster/issues/7))
- `Orders::get_by_payment_reference` and `DbOrder::find_by_payment_reference` for looking up an order by its payment reference, enabling webhook idempotency checks.
- `Orders::create_from_basket` now accepts an optional `payment_reference` argument.
- New default-settings migration seeding `stripe_webhook_secret` so every tenant has a row to read/update for verifying Stripe webhook signatures. ([#6](https://github.com/stec-ug-haftungsbeschrankt/shopster/issues/6))

### Migrations
- `2026-06-29-000000_stripe_webhook_secret`
- `2026-06-29-010000_orders_payment_reference`
