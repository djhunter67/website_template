//! Initialize and return a connection to the ``SQLite`` database.
use std::{sync::Arc, time::Duration};

use crate::settings::Settings;
use actix_web::web::Data;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tracing::{info, instrument};

#[must_use]
#[instrument(
    name = "Establishing a connection to the SQLite database",
    level = "info",
    skip(settings, manager)
)]
/// # Returns
///   - Returns a `Pool` of `SqliteConnectionManager` to the sqlite db if successful
///
/// # Arguments
///   - `settings` - The settings for the application
///
/// # Panics
///   - Panics if the pool cannot be created
///
/// Initialize and return a connection to the ``SQLite`` database.
pub fn establish_connection(
    settings: &Settings,
    manager: Data<SqliteConnectionManager>,
) -> Pool<SqliteConnectionManager> {
    info!("Establishing a connection to the SQLite database");
    r2d2::Pool::builder()
        .max_size(settings.sqlite.pool_size)
        .connection_timeout(Duration::from_secs(settings.sqlite.connection_timeout))
        .build(
            Arc::into_inner(manager.into_inner())
                .map_or_else(|| panic!("No Manager found"), |manager| manager),
        )
        .expect("Failed to create pool")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use rstest::rstest;
    use std::thread::spawn;

    use crate::settings::{self};

    use super::*;

    #[rstest]
    fn test_can_write_to_sqlite() {
        let manager = r2d2_sqlite::SqliteConnectionManager::file("test.db");
        let pool = establish_connection(&settings::get().unwrap(), Data::new(manager));
        let conn = pool.get().unwrap();

        // Test query
        conn.execute(
            "CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO test (name) VALUES (?)", ["Hello"])
            .unwrap();
        let result: String = conn
            .query_row("SELECT name FROM test WHERE id = 1", [], |row| row.get(0))
            .unwrap();

        drop(conn);

        // Remove the test database
        assert!(std::fs::remove_file("test.db").is_ok());

        // Test if the database was removed
        assert!(std::fs::metadata("test.db").is_err());

        assert_eq!(result, "Hello");
    }

    #[rstest]
    fn test_single_writer_sqlite() {
        // Create a connection pool
        let manager = r2d2_sqlite::SqliteConnectionManager::file("test_1.db");
        let pool = establish_connection(&settings::get().unwrap(), Data::new(manager));

        // Insert 5 items using a for loop
        let mut handles = vec![];

        // Create the table
        let conn = pool.get().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS test_table (id INTEGER PRIMARY KEY, value TEXT NOT NULL)",
            [],
        )
        .unwrap();

        drop(conn);

        for i in 0..5 {
            let pool = pool.clone();
            let handle = spawn(move || {
                let conn = pool.get().unwrap();

                // Insert an item
                let query = "INSERT INTO test_table (id, value) VALUES (?, ?)";
                conn.execute(query, (&i, &format!("item {i}"))).unwrap();
            });
            handles.push(handle);
        }

        // Wait for all writers to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Remove the test database
        assert!(std::fs::remove_file("test_1.db").is_ok());
        assert_eq!(get_count(&pool), 5);
    }

    #[rstest]
    fn test_write_and_read_sqlite() {
        let manager = r2d2_sqlite::SqliteConnectionManager::file("test_2.db");
        let pool = establish_connection(&settings::get().unwrap(), Data::new(manager));

        // Create table if not exists
        pool.get()
            .unwrap()
            .execute(
                "CREATE TABLE IF NOT EXISTS test (
            id INTEGER PRIMARY KEY,
            value TEXT NOT NULL
        )",
                [],
            )
            .unwrap();

        // Insert test value
        let test_value = "Test Value";
        pool.get()
            .unwrap()
            .execute("INSERT INTO test (value) VALUES (?)", [test_value])
            .unwrap();

        // Read the value back
        let result: String = pool
            .get()
            .unwrap()
            .query_row("SELECT value FROM test WHERE id = 1", [], |row| row.get(0))
            .unwrap();

        assert_eq!(result, test_value);

        drop(pool);
        // Remove the test database
        assert!(std::fs::remove_file("test_2.db").is_ok());
    }

    #[rstest]
    fn test_create_four_tables_sqlite() {
        // Create a connection to the new database
        let manager = r2d2_sqlite::SqliteConnectionManager::file("test_3.db");
        let pool = establish_connection(&settings::get().unwrap(), Data::new(manager));
        let conn = pool.get().unwrap();

        // Create four tables with arbitrary names
        let table_names = ["table1", "table2", "table3", "table4"];

        for name in &table_names {
            conn.execute(
                &format!("CREATE TABLE {name} (id INTEGER PRIMARY KEY, data TEXT NOT NULL)"),
                [],
            )
            .unwrap();
        }

        // Verify tables exist by querying sqlite_master
        let result: i32 = conn.query_row(
        "SELECT COUNT(name) FROM sqlite_master WHERE type='table' AND name IN (:1, :2, :3, :4)",
        [
	    &table_names[0],
	    &table_names[1],
	    &table_names[2],
	    &table_names[3],
	],

        |row| row.get(0),
    ).unwrap();

        assert_eq!(result, 4);

        drop(conn);
        // Remove the test database
        assert!(std::fs::remove_file("test_3.db").is_ok());
    }

    // Helper function to get the count of items
    fn get_count(pool: &Pool<SqliteConnectionManager>) -> usize {
        let conn = pool.get().unwrap();
        let query = "SELECT COUNT(*) FROM test_table";
        let count: usize = conn
            .query_row(query, [], |row| row.get(0))
            .unwrap_or_default();
        count
    }
}
