# Documentation Guide

Welcome to the Shopster documentation! This guide helps you navigate all available resources.

## Quick Navigation

### For Users

- **[README.md](../README.md)** - Overview, quick start, and common usage examples
- **[examples/](../examples/)** - Complete working examples
  - `basic_setup.rs` - Initialization and setup
  - `customer_workflow.rs` - Customer management
  - `shopping_workflow.rs` - Shopping cart operations
  - `product_management.rs` - Product catalog and inventory

### For Developers

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System design, data flow, and module responsibilities
- **[CONTRIBUTING.md](../CONTRIBUTING.md)** - Development setup, coding standards, and contribution guidelines
- **[Inline Documentation](#inline-documentation)** - API documentation in code

### API Documentation

Generate and view complete API documentation:

```bash
cargo doc --open
```

This opens the Rustdoc-generated API reference with:
- All public types and functions
- Method signatures and return types
- Usage examples from code comments
- Links between modules

## Documentation Structure

### README.md

The main entry point containing:

- **Overview** - What Shopster is and why you'd use it
- **Features** - Key capabilities
- **Quick Start** - Simple code example to get started
- **Usage Examples** - Code snippets showing:
  - Customer management
  - Shopping cart operations
  - Product management
  - Inventory management
- **Project Structure** - Directory organization
- **Setup Instructions** - Installation and configuration
- **Testing** - How to run tests
- **Common Patterns** - Best practices for using Shopster

### ARCHITECTURE.md

In-depth architecture documentation including:

- **Layered Architecture** - System design with ASCII diagrams
- **Multi-Tenancy Model** - How tenant isolation works
- **Core Modules** - Responsibilities and data structures for each module
- **Data Flow Examples** - Tracing operations through layers
- **Key Design Decisions** - Why certain choices were made
- **Integration Points** - External dependencies
- **Future Improvements** - Potential enhancements
- **Performance Considerations** - Optimization strategies

### CONTRIBUTING.md

Developer-focused documentation:

- **Setup** - Environment setup and prerequisites
- **Development Workflow** - How to make changes
- **Code Standards** - Style, documentation, testing requirements
- **File Organization** - Where to add new code
- **Common Tasks** - How to implement common features
- **Debugging** - Troubleshooting guide
- **Release Process** - How to publish changes

### examples/

Four complete, runnable examples demonstrating different workflows:

1. **basic_setup.rs** (~40 lines)
   - Initializing Tenet and Shopster
   - Creating database selector
   - Accessing domain handlers

2. **customer_workflow.rs** (~80 lines)
   - Creating customers
   - Authentication
   - Email verification
   - Searching
   - Listing

3. **shopping_workflow.rs** (~120 lines)
   - Creating baskets
   - Adding products
   - Calculating totals
   - Merging baskets

4. **product_management.rs** (~150 lines)
   - Creating products
   - Managing inventory
   - Pricing
   - Product catalog

Run examples with:
```bash
cargo run --example basic_setup
cargo run --example customer_workflow
cargo run --example shopping_workflow
cargo run --example product_management
```

## Inline Documentation

### Module-Level Documentation

Each module starts with doc comments explaining:
- Module purpose
- Main responsibilities
- Quick example usage

```rust
//! Customer management and authentication.
//!
//! This module handles customer CRUD operations, authentication, ...
//!
//! # Example
//!
//! ```ignore
//! let customers = shopster.customers(tenant_id)?;
//! let customer = customers.insert(&Customer { ... })?;
//! ```
```

### Type Documentation

All public types have doc comments with:
- Purpose and usage
- Field descriptions
- Example scenarios

```rust
/// A customer in the shop system.
///
/// Represents a registered customer with authentication credentials,
/// contact information, and account metadata.
pub struct Customer {
    /// Unique customer identifier
    pub id: Uuid,
    /// Customer's email address
    pub email: String,
    // ... etc
}
```

### Function Documentation

All public functions document:
- What the function does
- Arguments and their types
- Return type and possible errors
- Usage example

```rust
/// Creates a new customer.
///
/// # Arguments
///
/// * `customer` - The customer to insert
///
/// # Returns
///
/// `Ok(Customer)` - The created customer
/// `Err(ShopsterError)` - If creation fails
///
/// # Example
///
/// ```ignore
/// let customer = customers.insert(&customer)?;
/// ```
pub fn insert(&self, customer: &Customer) -> Result<Customer, ShopsterError> {
```

## Error Documentation

The `error.rs` module documents all possible errors:

- `TenetError` - Tenant service errors
- `SerializationError` - JSON parsing errors
- `DatabaseConnectionError` - Connection pool errors
- `DatabaseError` - SQL/Diesel errors
- `TenantNotFoundError` - Tenant not configured
- `TenantStorageNotFound` - No database for tenant
- `PasswordHashingError` - Password hashing failed
- `InvalidOperationError` - Invalid state or parameters
- `AuthenticationError` - Authentication failure

## Architecture Diagrams

### Layered Architecture

```
Public API (Shopster & Domain Handlers)
           ↓
Domain Layer (Customers, Products, Orders, etc.)
           ↓
Database Layer (postgresql/)
           ↓
External Services (PostgreSQL, Tenet, r2d2)
```

### Multi-Tenancy

```
Shopster Instance
    ↓
DatabaseSelector (Singleton)
    ├─ Tenant 1 → Connection Pool 1
    ├─ Tenant 2 → Connection Pool 2
    └─ Tenant 3 → Connection Pool 3
```

## Documentation Standards

### When Writing Documentation

1. **File Structure Changes** - Update README.md and ARCHITECTURE.md
2. **New Modules** - Add module doc comments with overview and example
3. **New Functions** - Document parameters, return type, errors, and example
4. **Complex Logic** - Add inline comments explaining "why"
5. **Breaking Changes** - Update guides and examples

### Documentation Quality Checklist

- [ ] All public items have doc comments
- [ ] Doc comments include usage examples
- [ ] Examples are marked as `ignore` if not runnable in tests
- [ ] Error cases are documented
- [ ] Complex algorithms have explanatory comments
- [ ] README examples match actual API
- [ ] ARCHITECTURE.md reflects current design

## Viewing Documentation

### Local Documentation

```bash
# Generate and view in browser
cargo doc --open

# Generate without opening
cargo doc --no-deps

# View specific crate documentation
cargo doc --package stec_shopster --open
```

### Documentation Features

The generated HTML documentation includes:
- Full API with types and functions
- Doc comment text with formatting
- Links between related types
- Examples from doc comments
- Search functionality
- Global search

## Contributing to Documentation

See [CONTRIBUTING.md](../CONTRIBUTING.md) for:
- Code documentation standards
- How to add examples
- Documentation review process

## External References

- **Rust Book** - https://doc.rust-lang.org/book/
- **Diesel Guide** - https://diesel.rs/
- **PostgreSQL Docs** - https://www.postgresql.org/docs/
- **Tenet Project** - See project repository

## Quick Reference

### Common Operations

**Get started:**
```bash
cargo run --example basic_setup
```

**View API docs:**
```bash
cargo doc --open
```

**Understand architecture:**
```bash
# Read ARCHITECTURE.md
cat docs/ARCHITECTURE.md
```

**Set up development:**
```bash
# See CONTRIBUTING.md
cat CONTRIBUTING.md
```

**See real examples:**
```bash
ls examples/
```

## FAQ

### Where do I find usage examples?
Check `README.md` for inline examples or `examples/` directory for complete working code.

### How do I understand the code structure?
Read `ARCHITECTURE.md` for system design and module responsibilities.

### How do I add a new feature?
Follow the workflow in `CONTRIBUTING.md`.

### Where's the API reference?
Run `cargo doc --open` to view complete API documentation.

### How do I troubleshoot issues?
See the debugging section in `CONTRIBUTING.md`.

### What's the performance model?
See "Performance Considerations" in `ARCHITECTURE.md`.

## Documentation Maintenance

Documentation is maintained alongside code:
- Update docs when changing APIs
- Keep examples in sync with code
- Review docs in pull requests
- Verify doc examples compile

---

**Last Updated**: June 2026
**Shopster Version**: 0.2.19+

