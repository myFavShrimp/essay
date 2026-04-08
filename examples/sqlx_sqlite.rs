//! SQLx + SQLite async example — run tests with:
//!   cargo test --example sqlx_sqlite

use sqlx::SqlitePool;

fn main() {
    println!("Run tests with: cargo test --example sqlx_sqlite");
}

#[cfg(test)]
async fn sqlite_pool() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            is_admin BOOLEAN NOT NULL DEFAULT FALSE
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
}

#[essay::essay(
    test_attr = tokio::test,
    cases(
        "create_user" => (sqlite_pool().await, "alice"),
    ),
)]
pub async fn create_user(pool: SqlitePool, username: &str) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>("INSERT INTO users (username) VALUES (?) RETURNING *")
        .bind(username)
        .fetch_one(&pool)
        .await
}

#[essay::essay(
    test_attr = tokio::test,
    cases(
        "list_admins" => (sqlite_pool().await),
    ),
)]
pub async fn list_admins(pool: SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE is_admin = TRUE")
        .fetch_all(&pool)
        .await
}

#[cfg(test)]
fn assert_sqlx_error(result: Result<User, sqlx::Error>) {
    assert!(result.is_err(), "expected error but got: {:?}", result);
}

#[essay::essay(
    test_attr = tokio::test,
    cases(
        "duplicate_username" => (sqlite_pool().await, "bob", "bob") -> assert_sqlx_error,
    ),
)]
pub async fn insert_duplicate(
    pool: SqlitePool,
    name1: &str,
    name2: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query("INSERT INTO users (username) VALUES (?)")
        .bind(name1)
        .execute(&pool)
        .await?;
    sqlx::query_as::<_, User>("INSERT INTO users (username) VALUES (?) RETURNING *")
        .bind(name2)
        .fetch_one(&pool)
        .await
}

#[essay::essay(
    test_attr = tokio::test,
    cases(
        "missing_table" => (sqlite_pool().await) -> assert_sqlx_error,
    ),
)]
pub async fn query_missing_table(pool: SqlitePool) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM no_such_table LIMIT 1")
        .fetch_one(&pool)
        .await
}
