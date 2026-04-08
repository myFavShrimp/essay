# essay

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/essay.svg
[crates-url]: https://crates.io/crates/essay
[docs-badge]: https://img.shields.io/docsrs/essay/latest.svg?logo=docsdotrs&label=docs.rs
[docs-url]: https://docs.rs/essay
[actions-badge]: https://github.com/myFavShrimp/essay/actions/workflows/ci.yml/badge.svg
[actions-url]: https://github.com/myFavShrimp/essay/actions/workflows/ci.yml

Generates tests for functions from attribute macros.

The crate is runtime-agnostic and knows nothing about databases, HTTP clients or async runtimes.

I built this crate to get [sqlx](https://crates.io/crates/sqlx)-like checked queries for other database clients.

## Usage

```rust
#[essay::essay(cases(
    "insert_cpu" => (duckdb_connection(), "cpu", 78.5),
    "insert_memory" => (duckdb_connection(), "memory", 45.2) -> check_metric_name_memory,
))]
fn insert_metric(conn: Connection, name: &str, val: f64) -> Result<Metric> {
    // ...
}
```

The generated test cases can be run via `cargo test`. See [examples](https://github.com/myFavShrimp/essay/tree/main/examples) for complete use cases.

## Async

Async is auto-detected from the function signature. `test_attr` can be used to set the test runtime.

```rust
#[essay::essay(
    test_attr = tokio::test,
    cases(
        "create_user" => (get_pool().await, "alice"),
    ),
)]
async fn create_user(pool: PgPool, username: &str) -> Result<User> {
    // ...
}
```

## Attribute syntax

```rust
#[essay::essay(
    test_attr = tokio::test,           // Custom test attribute (default is `test`)
    cases(
        "name" => (arg1) -> assert_fn, // custom assert function
        "name2" => (arg1, arg2),       // default is_ok() assertion
    )
)]
```
