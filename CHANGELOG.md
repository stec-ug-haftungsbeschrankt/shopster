# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0]

### Added
- `orders::Order` and `DbOrder` now have a `payment_reference` field (nullable, unique) for storing an external payment reference such as a Stripe Checkout Session ID or PaymentIntent ID. ([#7](https://github.com/stec-ug-haftungsbeschrankt/shopster/issues/7))
- `Orders::get_by_payment_reference` and `DbOrder::find_by_payment_reference` for looking up an order by its payment reference, enabling webhook idempotency checks.
- `Orders::create_from_basket` now accepts an optional `payment_reference` argument.
- New default-settings migration seeding `stripe_webhook_secret` so every tenant has a row to read/update for verifying Stripe webhook signatures. ([#6](https://github.com/stec-ug-haftungsbeschrankt/shopster/issues/6))

### Migrations
- `2026-06-29-000000_stripe_webhook_secret`
- `2026-06-29-010000_orders_payment_reference`
