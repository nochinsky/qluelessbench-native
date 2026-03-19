//! Database benchmark tests.
//!
//! Tests SQLite CRUD operations, indexed queries, and bulk inserts.

use anyhow::Result;
use rayon::prelude::*;
use rusqlite::{params, Connection};
use std::time::Instant;

use crate::benchmarks::base::{
    calculate_category_score, get_parallel_workers, run_with_iterations, BaseBenchmark,
};
use crate::results::CategoryResult;

/// Database benchmark.
pub struct DatabaseBenchmark {
    /// If true, run tests in parallel (multi-core mode)
    multi_core: bool,
}

impl DatabaseBenchmark {
    /// Create a new DatabaseBenchmark.
    pub fn new() -> Self {
        DatabaseBenchmark { multi_core: false }
    }

    /// Create a new DatabaseBenchmark for multi-core testing.
    pub fn new_multi_core() -> Self {
        DatabaseBenchmark { multi_core: true }
    }

    /// Create a test database connection.
    fn create_connection() -> Result<Connection> {
        let conn = Connection::open(":memory:")?;

        // Create users table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT NOT NULL,
                age INTEGER,
                city TEXT
            )",
            [],
        )?;

        // Create index
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)",
            [],
        )?;

        Ok(conn)
    }

    /// Test bulk inserts.
    fn test_bulk_insert() -> Result<f64> {
        let mut conn = Self::create_connection()?;
        let num_records = 1000;

        let start = Instant::now();
        let transaction = conn.transaction()?;
        for i in 0..num_records {
            transaction.execute(
                "INSERT INTO users (name, email, age, city) VALUES (?1, ?2, ?3, ?4)",
                params![
                    format!("User {}", i),
                    format!("user{}@example.com", i),
                    20 + (i % 50),
                    format!("City {}", i % 100)
                ],
            )?;
        }
        transaction.commit()?;
        let duration = start.elapsed().as_secs_f64();

        Ok(num_records as f64 / duration)
    }

    /// Test search queries.
    fn test_search() -> Result<f64> {
        let mut conn = Self::create_connection()?;

        // Pre-populate with data (outside timed region)
        let transaction = conn.transaction()?;
        for i in 0..5000 {
            transaction.execute(
                "INSERT INTO users (name, email, age, city) VALUES (?1, ?2, ?3, ?4)",
                params![
                    format!("User {}", i),
                    format!("user{}@example.com", i),
                    20 + (i % 50),
                    format!("City {}", i % 100)
                ],
            )?;
        }
        transaction.commit()?;

        // Test search (only this is timed)
        let start = Instant::now();
        let mut stmt = conn.prepare("SELECT * FROM users WHERE age > ?1")?;
        let rows: Vec<i64> = stmt
            .query_map(params![30], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        let count = rows.len();
        let duration = start.elapsed().as_secs_f64();

        Ok(count as f64 / duration)
    }

    /// Test updates.
    fn test_updates() -> Result<f64> {
        let mut conn = Self::create_connection()?;

        // Pre-populate
        let transaction = conn.transaction()?;
        for i in 0..1000 {
            transaction.execute(
                "INSERT INTO users (name, email, age, city) VALUES (?1, ?2, ?3, ?4)",
                params![
                    format!("User {}", i),
                    format!("user{}@example.com", i),
                    20 + (i % 50),
                    format!("City {}", i % 100)
                ],
            )?;
        }
        transaction.commit()?;

        // Test updates
        let start = Instant::now();
        let transaction = conn.transaction()?;
        for i in 0..500 {
            transaction.execute(
                "UPDATE users SET age = ?1 WHERE id = ?2",
                params![40, i + 1],
            )?;
        }
        transaction.commit()?;
        let duration = start.elapsed().as_secs_f64();

        Ok(500.0 / duration)
    }

    /// Test deletes.
    fn test_deletes() -> Result<f64> {
        let mut conn = Self::create_connection()?;

        // Pre-populate
        let transaction = conn.transaction()?;
        for i in 0..1000 {
            transaction.execute(
                "INSERT INTO users (name, email, age, city) VALUES (?1, ?2, ?3, ?4)",
                params![
                    format!("User {}", i),
                    format!("user{}@example.com", i),
                    20 + (i % 50),
                    format!("City {}", i % 100)
                ],
            )?;
        }
        transaction.commit()?;

        // Test deletes
        let start = Instant::now();
        let transaction = conn.transaction()?;
        for i in 0..500 {
            transaction.execute("DELETE FROM users WHERE id = ?1", params![i + 1])?;
        }
        transaction.commit()?;
        let duration = start.elapsed().as_secs_f64();

        Ok(500.0 / duration)
    }

    /// Test indexed lookup.
    fn test_indexed_lookup() -> Result<f64> {
        let mut conn = Self::create_connection()?;

        // Pre-populate
        let transaction = conn.transaction()?;
        for i in 0..5000 {
            transaction.execute(
                "INSERT INTO users (name, email, age, city) VALUES (?1, ?2, ?3, ?4)",
                params![
                    format!("User {}", i),
                    format!("user{}@example.com", i),
                    20 + (i % 50),
                    format!("City {}", i % 100)
                ],
            )?;
        }
        transaction.commit()?;

        // Test indexed lookup
        let start = Instant::now();
        let mut stmt = conn.prepare("SELECT * FROM users WHERE email = ?1")?;
        let _result: Option<(String, String, i32, String)> = stmt
            .query_row(params!["user100@example.com"], |row| {
                Ok((row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
            })
            .ok();
        let duration = start.elapsed().as_secs_f64();

        Ok(1.0 / duration)
    }

    /// Test parallel bulk inserts.
    /// Throughput model: Each connection inserts the FULL records (1000).
    /// N connections insert N× the records in roughly the same time = N× speedup.
    fn test_parallel_bulk_insert(num_connections: usize, num_records: usize) -> Result<f64> {
        let start = Instant::now();

        (0..num_connections)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                let mut conn = Self::create_connection()?;
                let transaction = conn.transaction()?;
                for i in 0..num_records {
                    transaction.execute(
                        "INSERT INTO users (name, email, age, city) VALUES (?1, ?2, ?3, ?4)",
                        params![
                            format!("User {}", i),
                            format!("user{}@example.com", i),
                            20 + (i % 50),
                            format!("City {}", i % 100)
                        ],
                    )?;
                }
                transaction.commit()?;
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_connections * num_records completed
        Ok((num_connections * num_records) as f64 / duration)
    }

    /// Test parallel search queries.
    /// Throughput model: Each connection searches the FULL dataset (5000 records).
    /// N connections search N× the records in roughly the same time = N× speedup.
    fn test_parallel_search(num_connections: usize, num_records: usize) -> Result<f64> {
        let start = Instant::now();

        (0..num_connections)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                let mut conn = Self::create_connection()?;

                // Pre-populate with full dataset
                let transaction = conn.transaction()?;
                for i in 0..num_records {
                    transaction.execute(
                        "INSERT INTO users (name, email, age, city) VALUES (?1, ?2, ?3, ?4)",
                        params![
                            format!("User {}", i),
                            format!("user{}@example.com", i),
                            20 + (i % 50),
                            format!("City {}", i % 100)
                        ],
                    )?;
                }
                transaction.commit()?;

                // Test search on this connection's full dataset
                let mut stmt = conn.prepare("SELECT * FROM users WHERE age > ?1")?;
                let _count: usize = stmt.query_row(params![30], |row| row.get(0))?;
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_connections tasks completed
        Ok(num_connections as f64 / duration)
    }

    /// Test parallel updates.
    /// Throughput model: Each connection updates the FULL records (500 updates).
    /// N connections update N× the records in roughly the same time = N× speedup.
    fn test_parallel_updates(num_connections: usize, num_updates: usize) -> Result<f64> {
        let start = Instant::now();

        (0..num_connections)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                let mut conn = Self::create_connection()?;

                // Pre-populate
                let transaction = conn.transaction()?;
                for i in 0..1000 {
                    transaction.execute(
                        "INSERT INTO users (name, email, age, city) VALUES (?1, ?2, ?3, ?4)",
                        params![
                            format!("User {}", i),
                            format!("user{}@example.com", i),
                            20 + (i % 50),
                            format!("City {}", i % 100)
                        ],
                    )?;
                }
                transaction.commit()?;

                // Test updates on full set
                let transaction = conn.transaction()?;
                for i in 0..num_updates {
                    transaction.execute(
                        "UPDATE users SET age = ?1 WHERE id = ?2",
                        params![40, i + 1],
                    )?;
                }
                transaction.commit()?;
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_connections * num_updates completed
        Ok((num_connections * num_updates) as f64 / duration)
    }

    /// Test parallel deletes.
    /// Throughput model: Each connection deletes the FULL records (500 deletes).
    /// N connections delete N× the records in roughly the same time = N× speedup.
    fn test_parallel_deletes(num_connections: usize, num_deletes: usize) -> Result<f64> {
        let start = Instant::now();

        (0..num_connections)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                let mut conn = Self::create_connection()?;

                // Pre-populate
                let transaction = conn.transaction()?;
                for i in 0..1000 {
                    transaction.execute(
                        "INSERT INTO users (name, email, age, city) VALUES (?1, ?2, ?3, ?4)",
                        params![
                            format!("User {}", i),
                            format!("user{}@example.com", i),
                            20 + (i % 50),
                            format!("City {}", i % 100)
                        ],
                    )?;
                }
                transaction.commit()?;

                // Test deletes on full set
                let transaction = conn.transaction()?;
                for i in 0..num_deletes {
                    transaction.execute("DELETE FROM users WHERE id = ?1", params![i + 1])?;
                }
                transaction.commit()?;
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_connections * num_deletes completed
        Ok((num_connections * num_deletes) as f64 / duration)
    }

    /// Test parallel indexed lookups.
    /// Throughput model: Each connection does the FULL lookup (5000 records, 1 lookup).
    /// N connections do N× the work in roughly the same time = N× speedup.
    fn test_parallel_indexed_lookup(num_connections: usize) -> Result<f64> {
        let start = Instant::now();

        (0..num_connections)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                let mut conn = Self::create_connection()?;

                // Pre-populate
                let transaction = conn.transaction()?;
                for i in 0..5000 {
                    transaction.execute(
                        "INSERT INTO users (name, email, age, city) VALUES (?1, ?2, ?3, ?4)",
                        params![
                            format!("User {}", i),
                            format!("user{}@example.com", i),
                            20 + (i % 50),
                            format!("City {}", i % 100)
                        ],
                    )?;
                }
                transaction.commit()?;

                // Test indexed lookup
                let mut stmt = conn.prepare("SELECT * FROM users WHERE email = ?1")?;
                let _result: Option<(String, String, i32, String)> = stmt
                    .query_row(params!["user100@example.com"], |row| {
                        Ok((row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
                    })
                    .ok();
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_connections tasks completed
        Ok(num_connections as f64 / duration)
    }
}

impl Default for DatabaseBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseBenchmark for DatabaseBenchmark {
    fn category_name(&self) -> &'static str {
        "Database"
    }

    fn weight(&self) -> f64 {
        1.2
    }

    fn run_all(&self, iterations: usize, warmup: usize, timeout: u64) -> Result<CategoryResult> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;

        // Reference values (operations per second)
        // Same reference values used for both single-core and multi-core modes
        let (insert_ref, search_ref, update_ref, delete_ref, indexed_ref) =
            (10000.0, 5000.0, 5000.0, 5000.0, 10000.0);

        if self.multi_core {
            // Multi-core: parallel database operations with SAME total work as single-core
            let num_workers = get_parallel_workers();

            let test_fn = || Self::test_parallel_bulk_insert(num_workers, 1000);
            let result = run_with_iterations(
                test_fn,
                &format!(
                    "Parallel Import Records (1000 records / {} connections)",
                    num_workers
                ),
                insert_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_search(num_workers, 5000);
            let result = run_with_iterations(
                test_fn,
                &format!(
                    "Parallel Search Users (5000 records / {} connections)",
                    num_workers
                ),
                search_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_updates(num_workers, 500);
            let result = run_with_iterations(
                test_fn,
                &format!(
                    "Parallel Update Profiles (500 updates / {} connections)",
                    num_workers
                ),
                update_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_deletes(num_workers, 500);
            let result = run_with_iterations(
                test_fn,
                &format!(
                    "Parallel Delete Records (500 deletes / {} connections)",
                    num_workers
                ),
                delete_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_indexed_lookup(num_workers);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Indexed Lookup ({} connections)", num_workers),
                indexed_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        } else {
            // Single-core tests
            // Test 1: Bulk Inserts
            let test_fn = || Self::test_bulk_insert();
            let result = run_with_iterations(
                test_fn,
                "Import Records",
                insert_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 2: Search
            let test_fn = || Self::test_search();
            let result = run_with_iterations(
                test_fn,
                "Search Users",
                search_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 3: Updates
            let test_fn = || Self::test_updates();
            let result = run_with_iterations(
                test_fn,
                "Update Profiles",
                update_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 4: Deletes
            let test_fn = || Self::test_deletes();
            let result = run_with_iterations(
                test_fn,
                "Delete Records",
                delete_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 5: Indexed Lookup
            let test_fn = || Self::test_indexed_lookup();
            let result = run_with_iterations(
                test_fn,
                "Indexed Lookup",
                indexed_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        }

        // Calculate category score
        let category_score = calculate_category_score(&results);

        Ok(CategoryResult {
            category: self.category_name().to_string(),
            score: category_score,
            duration: total_duration,
            weight: self.weight(),
            tests: results,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_category_name() {
        let benchmark = DatabaseBenchmark::new();
        assert_eq!(benchmark.category_name(), "Database");
    }

    #[test]
    fn test_database_weight() {
        let benchmark = DatabaseBenchmark::new();
        assert_eq!(benchmark.weight(), 1.2);
    }

    #[test]
    fn test_bulk_insert() {
        let result = DatabaseBenchmark::test_bulk_insert();
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }
}
