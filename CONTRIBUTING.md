# Contributing to minihub

Thank you for your interest in contributing to minihub!

## Getting started

1. Fork and clone the repository.
2. Install Rust stable (>= 1.85) via [rustup](https://rustup.rs/).
3. Install [just](https://github.com/casey/just): `cargo install just`
4. Install [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov): `cargo install cargo-llvm-cov`
5. Run `just check` to verify everything works.

## Development workflow

### Branching

- Work off `main`.
- Create a feature branch: `git checkout -b M1-T2-entity-types` (milestone-task-description).
- Keep commits focused — one logical change per commit.

### Before submitting a PR

Run the full check suite:

```bash
just check
```

This runs, in order:
1. `cargo fmt -- --check` — formatting
2. `cargo clippy --all-targets --all-features -- -D warnings` — linting
3. `cargo test --all` — tests

All three must pass.

### PR checklist

- [ ] Code compiles without warnings (`just clippy`)
- [ ] All existing tests pass (`just test`)
- [ ] New code has tests (aim for the milestone's coverage target)
- [ ] No `unwrap()` or `expect()` in non-test code (use proper error handling)
- [ ] Doc comments on public items
- [ ] No JavaScript added anywhere
- [ ] Dependency rules respected (see [ARCHITECTURE.md](docs/ARCHITECTURE.md)):
  - `domain` does not depend on `app` or any adapter
  - `app` does not depend on any adapter
  - No framework types (`axum::*`, `sqlx::*`) in `domain` or `app`
- [ ] TASKS.md updated if task is completed

### Coverage

Check coverage locally:

```bash
just cov        # terminal summary
just cov-html   # HTML report in target/llvm-cov/html/
```

### Working on milestones

See [TASKS.md](TASKS.md) for the full task breakdown. Each task has:
- A unique ID (e.g., `M1-T2`)
- A definition of done (DoD)
- Dependencies on other tasks

Pick a task, check its dependencies are met, implement it, and verify the DoD before submitting.

## Architecture

Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) before making structural changes. The hexagonal architecture and crate boundaries are load-bearing decisions.

## Coding rules

### Database queries

Do not use `sqlx::query!` or `sqlx::query_as!` compile-time macros. Use runtime queries with `.bind()` instead:

```rust
// Bad
let entity = sqlx::query_as!(Entity, "SELECT * FROM entities WHERE id = ?", id)
    .fetch_one(&pool)
    .await?;

// Good
let entity: Entity = sqlx::query_as("SELECT * FROM entities WHERE id = ?")
    .bind(id)
    .fetch_one(&pool)
    .await?;
```

### Wrapper pattern for SQLite adapters

When querying structures similar to domain entities, use a `Wrapper<T>` pattern to implement `FromRow` without polluting the domain with database concerns:

```rust
// In adapter_storage_sqlite_sqlx/src/lib.rs
struct Wrapper<T>(T);

impl<T> Wrapper<T> {
    fn maybe(value: Option<Self>) -> Option<T> {
        value.map(|s| s.0)
    }
}
```

Then implement `FromRow` for the wrapped domain type:

```rust
// In adapter_storage_sqlite_sqlx/src/repos.rs
use crate::Wrapper;

impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for Wrapper<Entity> {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(Self(Entity {
            id: row.try_get::<'r, String, _>(0).map(EntityId::from)?,
            name: row.try_get(1)?,
            state: row.try_get(2)?,
        }))
    }
}
```

Use `Wrapper::maybe` when fetching optional results:

```rust
sqlx::query_as(FIND_BY_ID)
    .bind(id)
    .fetch_optional(&self.pool)
    .await
    .map(Wrapper::maybe)
    .context("unable to find entity by id")
```

### No dynamic dispatch

Avoid `dyn` trait objects. Use generics instead:

```rust
// Bad
fn process(handler: Box<dyn Handler>) { ... }

// Good
fn process<H: Handler>(handler: H) { ... }
```

### Test naming

Tests must follow the `should_<behavior>_when_<condition>` pattern:

```rust
// Bad
#[test]
fn test_toggle() { ... }

#[test]
fn toggle_works() { ... }

// Good
#[test]
fn should_toggle_state_when_entity_is_on() { ... }

#[test]
fn should_return_error_when_entity_not_found() { ... }
```

### Comments

Do not use enumerated comments. Explain logic inline when necessary:

```rust
// Bad
// 1. First, we validate the input
validate(&input)?;
// 2. Then, we save to database
db.save(&input)?;
// 3. Finally, we return the result
Ok(result)

// Good
validate(&input)?;
db.save(&input)?;
Ok(result)
```

Do not use separator comments (banner-style section dividers). Use doc comments and natural code organization instead:

```rust
// Bad
// =============================================================================
// Entities
// =============================================================================

pub struct Entity { ... }

// =============================================================================
// Devices
// =============================================================================

pub struct Device { ... }

// Good — group related items together, use doc comments when needed

/// An observable/controllable data point in the system.
pub struct Entity { ... }

/// A physical or virtual thing that exposes one or more entities.
pub struct Device { ... }
```

### String interpolation

When interpolating strings in error messages or logs, prefer the debug format `{value:?}` over wrapping with quotes `'{}'`:

```rust
// Bad
format!("entity '{}' not found", id)
format!("unable to find device '{}'", name)

// Good
format!("entity {id:?} not found")
format!("unable to find device {name:?}")
```

This approach:
- Automatically adds quotes around strings
- Properly escapes special characters
- Shows `None` vs empty string distinction for `Option<String>`
- Uses inline variable syntax for cleaner code

### Error variable naming

Do not use single-letter variable names for errors. Use `err` or `error` for readability:

```rust
// Bad
.map_err(|e| MiniHubError::Internal(format!("failed: {e}")))?;
Err(err) => match &err {

// Good
.map_err(|err| MiniHubError::Internal(format!("failed: {err}")))?;
Err(error) => match &error {
```

### Avoid cloning

Avoid unnecessary `.clone()` calls. Prefer borrowing, moving, or using references:

```rust
// Bad
let name = user.name.clone();
process(&name);

// Good
let name = &user.name;
process(name);
```

If cloning is necessary, consider:
- Using `Arc<T>` for shared ownership
- Restructuring code to avoid the need for cloning
- Using `Cow<'_, T>` for conditional ownership

### HTTP handler response types

Handlers should return explicit response enums instead of `impl IntoResponse`. This makes the possible responses clear and type-safe:

```rust
// Bad — opaque return type
pub async fn handle(...) -> impl IntoResponse {
    if error {
        return (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response();
    }
    Html(body).into_response()
}

// Good — explicit enum with all possible responses
pub enum HomeResponse {
    Redirect(Redirect),
    Page(Html<String>),
}

impl IntoResponse for HomeResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Redirect(r) => r.into_response(),
            Self::Page(p) => p.into_response(),
        }
    }
}

pub async fn handle(...) -> Result<HomeResponse, ErrorPage> {
    // Handler logic returning Result
}
```

### Batch loading for repositories

Avoid N+1 query patterns. When loading related data for multiple entities, use batch loading methods instead of looping:

```rust
// Bad — N+1 queries
let devices = device_repo.list_by_area(area_id).await?;
for device in &devices {
    let entities = entity_repo.list_by_device(device.id).await?;
}

// Good — batch loading with a single query
let devices = device_repo.list_by_area(area_id).await?;
let device_ids: Vec<_> = devices.iter().map(|d| d.id).collect();
let entities = entity_repo.list_by_devices(&device_ids).await?;
```

### Service configuration pattern

Services should be built using a `Config` struct with `from_env()` and `build()` methods. Do not use `new()`, `connect()`, or similar constructors directly:

```rust
// Bad
let pool = SqlitePool::connect(&url).await?;
let server = HttpServer::new(address, port);

// Good
struct Config {
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("MINIHUB_DATABASE_URL")?,
        })
    }

    pub fn build(self) -> anyhow::Result<SqlitePool> {
        todo!()
    }
}

// Usage in binary
let config = Config::from_env()?;
let pool = config.build()?;
```

This pattern ensures:
- Configuration is explicit and testable
- Environment variable parsing is centralized in `from_env()`
- Service construction logic is encapsulated in `build()`
- Consistent API across all adapters

### No JavaScript

Do not use JavaScript in the dashboard. Use server-rendered HTML with forms (PRG pattern) and `<meta http-equiv="refresh">` for live updates. No JS, no WASM-requiring-JS, no npm.

### Error types

Use `thiserror` with typed source errors and `#[from]` conversion. Do not use `String` as the inner type — typed errors preserve the original error chain and produce better traces:

```rust
// Bad — String erases the original error type
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Storage error: {0}")]
    Storage(String),
}

// Usage loses the source:
.map_err(|err| MyError::Storage(err.to_string()))?;
```

```rust
// Good — typed source error with #[from]
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Storage error")]
    Storage(#[from] StorageError),
}

// Usage preserves the source:
let result = do_storage_thing()?; // auto-converts via From
```

Do not include the source error in the `#[error("...")]` message (e.g., `{0}`). The source error is already part of the error chain and will appear in the trace — duplicating it in the display message adds noise:

```rust
// Bad — duplicates the source in the display message
#[error("Validation error: {0}")]
Validation(#[from] ValidationError),

// Good — source is in the chain, not repeated
#[error("Validation error")]
Validation(#[from] ValidationError),
```

### Prefer `Default` over `new()` without arguments

When a constructor takes no arguments, implement `Default` instead of `new()`:

```rust
// Bad
impl MyStruct {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}

// Good
impl Default for MyStruct {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}
```

This integrates with Rust idioms like `#[derive(Default)]` and `Option::unwrap_or_default()`.

### Code quality

- Run `just fmt` before committing
- Run `just clippy` and fix all warnings
- Prefer editing existing files over creating new ones
- Maintain at least the milestone's coverage target using `cargo llvm-cov`
- Use `thiserror` for error types (see [Error types](#error-types) above)
- Write doc comments for all public items

## Reporting issues

Open an issue with:
- What you expected to happen
- What actually happened
- Steps to reproduce
- Rust version (`rustc --version`)
