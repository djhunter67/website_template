//! Initialize and return a connection to the ``SQLite`` database.
use std::sync::{Arc, Mutex};

use crate::settings::Settings;
use actix_web::web::Data;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use tracing::instrument;

///Wrapper for the `SqliteConnectionManager` to be used with `r2d2`
pub struct SQLiteConnectionManagerWrapper(Arc<Mutex<SqliteConnectionManager>>);

impl r2d2::ManageConnection for SQLiteConnectionManagerWrapper {
    type Connection = rusqlite::Connection;
    type Error = rusqlite::Error;

    #[instrument(level = "info", name = "Connecting to the SQLite database", skip(self))]
    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let manager = self
            .0
            .lock()
            .expect("No database connection found in the pool.")
            .connect()?;
        Ok(manager)
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        conn.execute("SELECT 1", rusqlite::params![])?;
        Ok(())
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        conn.execute("SELECT 1", rusqlite::params![]).is_err()
    }
}

#[must_use]
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
    manager: &Data<Arc<Mutex<SqliteConnectionManager>>>,
) -> Pool<SQLiteConnectionManagerWrapper> {
    let pool = r2d2::Pool::builder()
        .max_size(settings.sqlite.pool_size)
        .build(SQLiteConnectionManagerWrapper(manager.get_ref().clone()))
        .expect("Failed to create pool");

    pool
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::settings::{self};

    use super::*;

    #[test]
    fn test_establish_connection() {
        let settings = settings::get().expect("Failed to load settings");

        // Create a mock SQLite connection manager for testing purposes
        let mock_manager = Arc::new(Mutex::new(SqliteConnectionManager::file("test.db")));

        // Call the establish_connection function with the mock settings and manager
        let pool = establish_connection(&settings, &Data::new(mock_manager));

        // Assert that the returned value is a valid r2d2::Pool
        assert!(pool.test_on_check_out());

        // Drop the pool
        drop(pool);

        // Delete the database file
        std::fs::remove_file("test.db").expect("Failed to delete file");
    }

    #[test]
    fn test_db_is_created() {
        let settings = settings::get().expect("Failed to load settings");

        // Create a mock SQLite connection manager for testing purposes
        let mock_manager = Arc::new(Mutex::new(SqliteConnectionManager::file("test.db")));

        // Call the establish_connection function with the mock settings and manager
        let _pool = establish_connection(&settings, &Data::new(mock_manager));

        // Assert that the database file was created
        assert!(std::path::Path::new("test.db").exists());

        // Delete the database file
        std::fs::remove_file("test.db").expect("Failed to delete file");
    }

    #[test]
    fn test_if_can_write_to_db() {
        let settings = settings::get().expect("Failed to load settings");

        // Create a mock SQLite connection manager for testing purposes
        let mock_manager = Arc::new(Mutex::new(SqliteConnectionManager::file("test.db")));

        // Call the establish_connection function with the mock settings and manager
        let pool = establish_connection(&settings, &Data::new(mock_manager));

        // Get a connection from the pool
        let conn = pool.get().expect("Failed to get connection");

        // Create a table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY, name TEXT)",
            rusqlite::params![],
        )
        .expect("Failed to create table");

        // Insert a row into the table
        assert!(conn
            .execute(
                "INSERT INTO test (name) VALUES (?)",
                rusqlite::params!["test"],
            )
            .is_ok());

        // Drop the table
        conn.execute("DROP TABLE test", rusqlite::params![])
            .expect("Failed to drop table");
    }

    #[test]
    #[ignore]
    fn test_if_can_read_from_db() {
        let settings = settings::get().expect("Failed to load settings");

        // Create a mock SQLite connection manager for testing purposes
        let mock_manager = Arc::new(Mutex::new(SqliteConnectionManager::file("test.db")));

        // Call the establish_connection function with the mock settings and manager
        let pool = establish_connection(&settings, &Data::new(mock_manager));

        // Get a connection from the pool
        let conn = pool.get().expect("Failed to get connection");

        // Create a table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY, name TEXT)",
            rusqlite::params![],
        )
        .expect("Failed to create table");

        // Insert a row into the table
        conn.execute(
            "INSERT INTO test (name) VALUES (?)",
            rusqlite::params!["test"],
        )
        .expect("Failed to insert row");

        // Query the table
        let mut stmt = conn
            .prepare("SELECT name FROM test")
            .expect("Failed to prepare statement");
        let rows = stmt
            .query_map(rusqlite::params![], |row| row.get::<_, String>(0))
            .expect("Failed to query table");

        // Assert that the row was inserted
        for name in rows {
            assert_eq!(name.unwrap(), "test");
        }

        // Drop the table
        conn.execute("DROP TABLE test", rusqlite::params![])
            .expect("Failed to drop table");
    }
}
