# Shopster

Database Layer for shop system with tenant support.

## Overview

Shopster is a Rust-based database abstraction layer designed for e-commerce systems with multi-tenant support. It provides a clean API for managing customers, products, orders, baskets, and shop settings using PostgreSQL as the backend database.

## Features

- **Multi-tenant Support**: Built-in tenant isolation for managing multiple shops
- **Comprehensive E-commerce Models**: Support for:
    - Customers and customer management
    - Products with tags and images
    - Shopping baskets
    - Order processing
    - Shop settings and configuration
- **PostgreSQL Backend**: Uses Diesel ORM for type-safe database interactions
- **Type-safe**: Leverages Rust's type system for compile-time guarantees

## Project Structure

The project is organized as follows:
- `src/postgresql/` - PostgreSQL-specific database implementations
- `src/baskets.rs` - Basket domain logic
- `src/customers.rs` - Customer domain logic
- `src/orders.rs` - Order domain logic
- `src/products.rs` - Product domain logic
- `src/settings.rs` - Settings domain logic
- `src/schema.rs` - Database schema definitions
- `src/error.rs` - Error types
- `migrations/` - Database migrations
- `tests/` - Integration tests

## Prerequisites

- Rust (latest stable version)
- Docker (for running tests)
- PostgreSQL

## Installation

Add Shopster to your Cargo.toml dependencies section.

## Configuration

Create an .env file in the project root with your database configuration.

## Database Setup

Run migrations using Diesel CLI:

    cargo install diesel_cli --no-default-features --features postgres
    diesel migration run

## Testing

We use unit/integration tests. In order to run them you need docker running and have cargo-nexttest installed. You can do this with:

    cargo install cargo-nextest --locked

To test run the tests, use the following command:

    cargo nextest run

For Test coverage use:

    cargo install cargo-llvm-cov
    cargo llvm-cov nextest

## CI/CD

This project uses GitHub Actions for continuous integration. The workflow is defined in .github/workflows/build.yml and runs automatically on push and pull requests.

## License

GPL-3.0-or-later
