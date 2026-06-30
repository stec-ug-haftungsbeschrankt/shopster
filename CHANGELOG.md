# Changelog

All notable changes to this project will be documented in this file.

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
