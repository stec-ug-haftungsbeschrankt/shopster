# Shopster Architecture

## Overview

Shopster is a multi-tenant e-commerce database layer written in Rust. It implements a layered architecture with clear separation of concerns and data isolation across tenants.

## Architectural Layers

```
┌─────────────────────────────────────────────────────────────┐
│                     Public API Layer                        │
│          (Shopster struct & Domain Module APIs)             │
│  ┌──────────┬──────────┬───────────┬────────────┬─────────┐ │
│  │Customers │ Products │   Orders  │  Baskets   │Settings │ │
│  │          │          │           │  Warehouse │         │ │
│  └──────────┴──────────┴───────────┴────────────┴─────────┘ │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                 Domain Layer                                │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Data structures, conversions, business logic        │  │
│  │  - Customer (authentication, profiles)               │  │
│  │  - Product (catalog management)                      │  │
│  │  - Order (processing, status tracking)               │  │
│  │  - Basket (cart operations)                          │  │
│  │  - Warehouse (inventory)                             │  │
│  │  - Settings (configuration)                          │  │
│  └──────────────────────────────────────────────────────┘  │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│              Database Abstraction Layer                     │
│                   (postgresql/)                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Diesel ORM operations, SQL queries                  │  │
│  │  - DbCustomer, DbProduct, DbOrder, etc.              │  │
│  │  - Tenant-aware database access                      │  │
│  │  - Connection pooling via r2d2                       │  │
│  └──────────────────────────────────────────────────────┘  │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│              Platform & External Services                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  PostgreSQL Database                                 │  │
│  │  Tenet (multi-tenant service)                        │  │
│  │  r2d2 Connection Pool                                │  │
│  │  Diesel ORM                                          │  │
│  └──────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

## Multi-Tenancy Model

### Tenant Isolation

Every operation in Shopster is tenant-aware:

```rust
// All domain handlers require a tenant_id
let customers = shopster.customers(tenant_id)?;
let products = shopster.products(tenant_id)?;
let orders = shopster.orders(tenant_id)?;
```

### Connection Management

- **DatabaseSelector**: Global singleton that maintains:
  - Tenant-to-Tenet metadata mapping
  - Per-tenant connection pool cache
  - Automatic database creation and migration

- **Connection Pooling**: Each tenant has its own r2d2 connection pool
  - Connections are lazily created on first access
  - Reduces overhead for single-tenant deployments
  - Enables efficient multi-tenant scaling

```rust
pub struct DatabaseSelector {
    tenants: Tenet,  // Tenant metadata service
    database_cache: HashMap<Uuid, Pool>  // Per-tenant pools
}
```

### Data Isolation

All SQL queries include tenant filtering:
- Implicit through table structure (tenant_id column or multi-schema)
- Explicit in WHERE clauses
- Enforced at the database layer

## Core Modules

### `lib.rs` - Entry Point

- **Shopster**: Main facade providing access to domain handlers
- **DatabaseSelector**: Connection and tenant management
- **aquire_database()**: Internal function for connection pooling

### `customers.rs` - Customer Management

**Responsibilities:**
- Customer CRUD operations
- Authentication (email/password verification)
- Password hashing with Argon2
- Email verification
- Customer search and pagination

**Key Structures:**
- `Customer`: Public domain model
- `Customers`: Handler with business logic
- `DbCustomer`: Database model (postgresql/)

**Operations:**
```rust
customers.get_all()
customers.get(id)
customers.find_by_email(email)
customers.insert(&customer)
customers.update(&customer)
customers.verify_email_password(email, password)
```

### `products.rs` - Product Catalog

**Responsibilities:**
- Product CRUD operations
- Pricing management
- Image and tag management
- Product search and filtering

**Key Structures:**
- `Product`: Product with all details
- `Price`: Pricing information (amount in cents, currency)
- `Products`: Handler

**Operations:**
```rust
products.get_all()
products.get(id)
products.insert(&product)
products.update(&product)
```

### `baskets.rs` - Shopping Cart

**Responsibilities:**
- Basket creation and deletion
- Adding/removing products
- Quantity management
- Basket merging
- Total calculation

**Key Structures:**
- `Basket`: Shopping cart
- `BasketProduct`: Product line item
- `BasketProductWithDetails`: Item with full product info
- `Baskets`: Handler

**Operations:**
```rust
baskets.add_basket()
baskets.add_product_to_basket(basket_id, product_id, quantity)
baskets.get_products_with_details(basket_id)
baskets.calculate_basket_total(basket_id)
baskets.merge_baskets(source, target)
```

### `orders.rs` - Order Processing

**Responsibilities:**
- Order CRUD operations
- Order status tracking and transitions
- Order item snapshots (frozen product state)
- Inventory reservation on status changes

**Key Structures:**
- `Order`: Complete order with items and addresses
- `OrderStatus`: Enum for order fulfillment lifecycle (New → Done, or cancelled at any non-terminal point)
- `PaymentStatus`: Enum for payment state (Pending, Paid, Failed, Refunded), tracked independently of `OrderStatus`
- `OrderItemSnapshot`: Historical product snapshot
- `Orders`: Handler

**Order Status Flow:**
```
New → InProgress → ReadyToShip → Shipping → Done
```
Any of `New`, `InProgress`, `ReadyToShip`, `Shipping` may also transition directly to the terminal `Cancelled` status (e.g. customer cancellation, stock unavailable, fraud check failure). No transition is valid out of `Done` or `Cancelled`. Cancelling a reserving order releases its warehouse reservation.

`PaymentStatus` is a separate axis from `OrderStatus` — fulfillment and payment progress independently of each other (e.g. an order can be `Cancelled` while `Paid`, awaiting refund, or `Shipping` while payment is still `Pending` for invoice/COD orders).

**Operations:**
```rust
orders.get_all()
orders.insert(&order)
orders.update(&order)                                  // fulfillment status transitions, validated
orders.update_payment_status(order_id, payment_status)  // payment status, independent of fulfillment
```

### `warehouse.rs` - Inventory Management

**Responsibilities:**
- Stock tracking (in_stock vs reserved)
- Inventory reservations
- Warehouse operations

**Key Structures:**
- `WarehouseItem`: Stock entry for a product
- `WarehouseItemDetails`: Item with product details
- `Warehouse`: Handler

**Available vs Reserved:**
```
Available = in_stock - reserved
```

**Operations:**
```rust
warehouse.get_all()
warehouse.insert(&item)
warehouse.apply_reserved_delta(product_id, delta)
```

### `settings.rs` - Configuration

**Responsibilities:**
- Shop configuration storage
- Key-value settings with type information
- Settings validation and management

**Key Structures:**
- `Setting`: Configuration entry
- `Settings`: Handler

**Operations:**
```rust
settings.get_all()
settings.get_by_title(key)
settings.insert(key, type, value)
settings.update_by_id(id, value)
```

### `error.rs` - Error Handling

**Error Types:**
- `TenetError`: Tenant service errors
- `SerializationError`: JSON errors
- `DatabaseConnectionError`: Connection pool errors
- `DatabaseError`: Diesel/SQL errors
- `TenantNotFoundError`: Tenant not configured
- `TenantStorageNotFound`: No database for tenant
- `PasswordHashingError`: Argon2 errors
- `InvalidOperationError`: Invalid state/params
- `AuthenticationError`: Auth failures

## Data Flow Examples

### Creating a Customer

```
1. User calls: customers.insert(&customer)
2. Domain Layer:
   - Converts Customer → DbCustomer
   - Passes to database layer
3. Database Layer:
   - Executes INSERT SQL with tenant_id
   - Returns created DbCustomer
4. Domain Layer:
   - Converts DbCustomer → Customer
   - Returns to user
```

### Adding Product to Basket

```
1. User calls: baskets.add_product_to_basket(basket_id, product_id, qty)
2. Database Layer:
   - Checks if product already in basket
3. If exists: UPDATE quantity
   If not: INSERT new basket_product row
4. Return entry ID
```

### Order Status Transition

```
1. User calls: orders.update_status(order_id, ReadyToShip)
2. Orders handler:
   - Checks if transition is valid
3. If status changes to reserve status:
   - Call warehouse.apply_reserved_delta()
   - Reserve inventory for order items
4. Update order.status in database
5. Return updated order
```

## Key Design Decisions

### 1. **Tenant-First Design**
Every operation explicitly requires a tenant ID, preventing accidental cross-tenant access.

### 2. **Domain Models as API**
Public API uses high-level domain models (Customer, Product, Order) not database models, hiding schema details and enabling schema evolution.

### 3. **Connection Caching Per Tenant**
Avoids per-request connection setup overhead while supporting multi-tenancy, implemented via `DatabaseSelector` singleton.

### 4. **Snapshots for Historical Data**
Order items capture product state at order time, preserved indefinitely even if product is later modified or deleted.

### 5. **Type-Safe Conversions**
Explicit `From` trait implementations between domain and database models ensure type safety at compile time.

### 6. **Lazy Initialization**
Databases are created on-demand with automatic migrations on first tenant access, reducing startup complexity.

## Integration Points

### Tenet Service
Shopster depends on `stec_tenet` for:
- Tenant metadata and configuration
- Tenant discovery by ID
- Storage connection strings

### PostgreSQL
- All data persisted to PostgreSQL
- Diesel ORM for type-safe queries
- Migrations embedded in binary

### r2d2
- Connection pooling
- Per-tenant pool isolation
- Automatic connection lifecycle management

## Future Improvements

Potential enhancements based on FIXME comments:

1. **Storage Selection Strategy**: Currently uses first storage for tenant; could support multiple storage types (primary/replica, geographic, etc.)
2. **Event System**: Add business events (OrderCreated, ProductUpdated, etc.)
3. **Caching Layer**: Add Redis/in-memory cache for frequently accessed data
4. **Audit Trail**: Capture all mutations with user/timestamp
5. **Advanced Inventory**: Support warehouse locations, serial numbers, expiration dates
6. **Payment Integration**: Support multiple payment providers
7. **Notification System**: Email, SMS, webhook support for order events

## Performance Considerations

### Connection Pooling
- Single connection pool per tenant
- Lazy initialization on first access
- Configurable pool size (defaults to r2d2 settings)

### Query Optimization
- Use Diesel's query builder for parameterized queries
- Some complex queries may benefit from raw SQL
- Consider indexes on tenant_id and common filters

### Scaling Strategies
- **Horizontal**: Multiple app instances share database (standard multi-tenant)
- **Vertical**: Larger connection pools for each tenant
- **Sharding**: Separate databases per tenant group

