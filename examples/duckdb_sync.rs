//! DuckDB sync example — run tests with:
//!   cargo test --example duckdb_sync

use duckdb::Connection;

fn main() {
    println!("Run tests with: cargo test --example duckdb_sync");
}

#[cfg(test)]
fn duckdb_conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();

    conn.execute_batch(
        "CREATE SEQUENCE metrics_seq START 1;
         CREATE TABLE metrics (
             id BIGINT DEFAULT nextval('metrics_seq') PRIMARY KEY,
             name VARCHAR NOT NULL,
             value DOUBLE NOT NULL
         );
         CREATE TABLE events (ts TIMESTAMP, kind VARCHAR);",
    )
    .unwrap();

    conn
}

#[derive(Debug)]
pub struct Metric {
    pub id: i64,
    pub name: String,
    pub value: f64,
}

#[cfg(test)]
fn check_metric_name_memory(result: duckdb::Result<Metric>) {
    let unwrapped = result.unwrap();

    assert_eq!(unwrapped.name, "memory");
}

#[cfg(test)]
fn assert_err(result: duckdb::Result<Metric>) {
    assert!(
        result.is_err(),
        "expected error but got: {:?}",
        result.unwrap()
    );
}

#[essay::essay(cases(
    "insert_cpu" => (duckdb_conn(), "cpu_usage", 78.5),
    "insert_memory" => (duckdb_conn(), "memory", 45.2) -> check_metric_name_memory,
))]
pub fn insert_metric(conn: Connection, name: &str, value: f64) -> duckdb::Result<Metric> {
    conn.query_row(
        "INSERT INTO metrics (name, value) VALUES (?, ?) RETURNING id, name, value",
        duckdb::params![name, value],
        |row| {
            Ok(Metric {
                id: row.get(0)?,
                name: row.get(1)?,
                value: row.get(2)?,
            })
        },
    )
}

#[essay::essay(cases(
    "select_from_nonexistent_table" => (duckdb_conn()) -> assert_err,
))]
pub fn query_missing_table(conn: Connection) -> duckdb::Result<Metric> {
    conn.query_row("SELECT id, name, value FROM no_such_table", [], |row| {
        Ok(Metric {
            id: row.get(0)?,
            name: row.get(1)?,
            value: row.get(2)?,
        })
    })
}

#[essay::essay(cases(
    "insert_into_nonexistent_column" => (duckdb_conn(), "cpu", 1.0) -> assert_err,
))]
pub fn insert_bad_column(conn: Connection, name: &str, value: f64) -> duckdb::Result<Metric> {
    conn.query_row(
        "INSERT INTO metrics (nonexistent_col, value) VALUES (?, ?) RETURNING id, name, value",
        duckdb::params![name, value],
        |row| {
            Ok(Metric {
                id: row.get(0)?,
                name: row.get(1)?,
                value: row.get(2)?,
            })
        },
    )
}
