# Contributing to Shopster

Thank you for your interest in contributing to Shopster! This document provides guidelines and information for contributors.

## Code of Conduct

Be respectful and constructive. We're building a welcoming community.

## Getting Started

### Prerequisites

- Rust 1.70+ (install from https://rustup.rs/)
- PostgreSQL 12+
- Docker (for running tests)
- `cargo-nextest` for testing (install with `cargo install cargo-nextest --locked`)

### Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/stec-ug-haftungsbeschrankt/shopster
cd shopster

# Install dependencies (skip for Rust projects, cargo handles this)
# but install test tools
cargo install cargo-nextest --locked
cargo install cargo-llvm-cov

# Create .env file for local development
cat > .env << EOF
DATABASE_URL=postgres://postgres:postgres@localhost/shopster_db
TENET_DATABASE_URL=postgres://postgres:postgres@localhost/tenet_db
RUST_LOG=info
EOF
```

## Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 2. Make Changes

Follow Rust conventions and code style. Key principles:

- **Rust Style**: Use `cargo fmt` to format code
- **Naming**: Use clear, descriptive names
- **Comments**: Document non-obvious logic, especially business rules
- **Types**: Prefer explicit types; let type inference work where readable
- **Errors**: Use `ShopsterError` for domain errors, propagate with `?`

### 3. Add Documentation

For all public APIs:

```rust
/// Brief description of what this does.
///
/// Longer explanation if needed.
///
/// # Arguments
///
/// * `param1` - Description
/// * `param2` - Description
///
/// # Returns
///
/// `Ok(T)` - Success description
/// `Err(ShopsterError)` - Error cases
///
/// # Example
///
/// ```ignore
/// let result = function(param1, param2)?;
/// ```
pub fn my_function(param1: Type1, param2: Type2) -> Result<ReturnType, ShopsterError> {
    // ...
}
```

### 4. Write Tests

- Add unit tests in the same file as the function
- Add integration tests in `tests/` for cross-module tests
- Test both success and error cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something_works() {
        let result = some_function();
        assert_eq!(result, expected_value);
    }

    #[test]
    fn test_error_case() {
        let result = some_function_that_fails();
        assert!(result.is_err());
    }
}
```

### 5. Run Tests Locally

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy

# Run tests
cargo nextest run

# Generate documentation
cargo doc --open
```

## Code Standards

### Lint and Format

```bash
# Format with rustfmt
cargo fmt

# Check formatting (CI requirement)
cargo fmt -- --check

# Lint with Clippy
cargo clippy

# Strict clippy checks
cargo clippy -- -D warnings
```

### Documentation

- All public items must have documentation
- Documentation should include examples for complex functions
- Keep docs up-to-date with code changes
- Use proper markdown syntax

### Testing

- Minimum 70% code coverage for new code
- All public APIs must have tests
- Include both happy path and error cases

```bash
# Generate coverage report
cargo llvm-cov nextest --html
open target/llvm-cov/html/index.html
```

## File Organization

### Adding a New Domain Module

If adding a new feature area (e.g., `reports.rs`):

1. Create `src/reports.rs` with public domain models
2. Create `src/postgresql/dbreports.rs` with database operations
3. Update `src/postgresql/mod.rs` to expose new module
4. Update `src/lib.rs` to export new module and add to Shopster
5. Add comprehensive docs and examples
6. Add integration tests in `tests/reports_tests.rs`

### Example Structure

```rust
// src/reports.rs
//! Report generation and analytics.

use uuid::Uuid;
use crate::error::ShopsterError;
use crate::postgresql::dbreports::DbReport;

/// A generated report with analytics data.
pub struct Report {
    pub id: i64,
    pub name: String,
    pub data: Vec<ReportRow>,
}

/// Handler for report operations.
pub struct Reports {
    tenant_id: Uuid,
}

impl Reports {
    pub fn new(tenant_id: Uuid) -> Self {
        Reports { tenant_id }
    }

    /// Generates a report for the tenant.
    pub fn generate(&self, report_type: &str) -> Result<Report, ShopsterError> {
        // Implementation
    }
}
```

## Database Migrations

For schema changes:

1. Create migration: `diesel migration generate migration_name`
2. Edit `migrations/TIMESTAMP_migration_name/up.sql` with schema changes
3. Edit `migrations/TIMESTAMP_migration_name/down.sql` with rollback
4. Update `src/schema.rs` with Diesel table! macros
5. Add tests for migration

```bash
# Create new migration
diesel migration generate add_user_profiles

# Run migrations
diesel migration run

# Revert latest migration
diesel migration redo
```

## Pull Request Process

### Before Submitting

- [ ] Code is formatted: `cargo fmt`
- [ ] No clippy warnings: `cargo clippy`
- [ ] All tests pass: `cargo nextest run`
- [ ] Documentation updated for public APIs
- [ ] Examples added for new features
- [ ] CHANGELOG.md updated (if applicable)

### PR Template

```markdown
## Description

Brief description of changes.

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Related Issues

Closes #ISSUE_NUMBER

## Testing

Describe testing performed.

## Checklist

- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] All tests pass
```

## Common Tasks

### Adding a Method to a Domain Handler

```rust
impl customers::Customers {
    /// Describe what this method does.
    ///
    /// # Example
    /// ...
    pub fn my_method(&self, param: Type) -> Result<Output, ShopsterError> {
        let db_result = DbCustomer::my_operation(self.tenant_id, param)?;
        Ok(transformation(&db_result))
    }
}
```

### Adding Error Cases

1. Add variant to `ShopsterError` enum in `error.rs`
2. Implement error message with `#[error(...)]`
3. Use with `Err(ShopsterError::NewError)`

```rust
#[derive(Error, Debug)]
pub enum ShopsterError {
    // ...existing errors...
    
    #[error("Customer not found")]
    CustomerNotFound,
    
    #[error("Invalid email format")]
    InvalidEmailFormat(String),
}
```

### Adding Database Operations

1. Add types to `postgresql/dbmodule.rs`
2. Implement CRUD operations
3. Keep queries tenant-aware
4. Add tests in test module

## Debugging

### Enable Logging

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# More verbose
RUST_LOG=shopster=trace cargo run
```

### Common Issues

**Connection pooling errors**: Check DATABASE_URL is correct and PostgreSQL is running

**Migration errors**: Verify migrations are in correct order, run `diesel migration redo`

**Compilation errors**: Run `cargo check` for detailed error messages

## Performance Tips

1. **Use connection pooling**: Don't create new connections per request
2. **Lazy initialization**: Load data only when needed
3. **Indexes**: Add database indexes for frequently filtered fields
4. **Query optimization**: Use Diesel's query builder to generate efficient SQL

## Documentation

### Update Documentation When

- Adding new public functions
- Changing function signatures
- Adding new modules
- Fixing bugs (update examples if applicable)
- Adding new use cases

### Documentation Format

Use rustdoc comments (`///`) for public items:

```rust
/// Brief one-liner.
///
/// Longer explanation with context.
///
/// # Errors
///
/// Returns `Err` if X happens.
```

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag
4. Push to repository
5. Publish to crates.io: `cargo publish`

## Questions?

- Open an issue for bugs or feature requests
- Check existing documentation in `docs/`
- Review similar code in the repository
- Ask in project discussions if unsure

Thank you for contributing to Shopster! 🎉

