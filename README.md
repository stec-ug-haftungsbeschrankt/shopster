# Shopster

Database Layer for shop system with tenant support.

## Testing

We use unit/integration tests. In order to run them you need `docker`running and have `cargo-nexttest` installed. You can do this with:

```bash
> cargo install cargo-nextest --locked
```

To test run the tests, use the following command:

```bash
> cargo nextest run
```

For Test coverage use:

```bash
> cargo install cargo-llvm-cov
> cargo llvm-cov nextest
```