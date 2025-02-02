//! Initialize and return a connection to the ``Redis`` database.

use std::{sync::Arc, time::Duration};

use crate::settings::Settings;

use actix_web::web::Data;

use r2d2::Pool;

use r2d2_redis::RedisConnectionManager;

use tracing::instrument;

#[must_use]
#[instrument(
    name = "Establishing a connection to the Redis database",
    level = "info",
    skip(settings, manager)
)]
/// # Returns
///   - Returns a `Pool` of `RedisConnectionManager` to the redis db if successful
///
/// # Arguments
///   - `settings` - The settings for the application
///
/// # Panics
///   - Panics if the pool cannot be created
///
/// Initialize and return a connection to the ``Redis`` database.
pub fn establish_connection(
    settings: &Settings,
    manager: Data<RedisConnectionManager>,
) -> Pool<RedisConnectionManager> {
    r2d2::Pool::builder()
        .max_size(settings.redis.pool_size)
        .connection_timeout(Duration::from_secs(
            settings.redis.pool_timeout_seconds.into(),
        ))
        .build(
            Arc::into_inner(manager.into_inner())
                .map_or_else(|| panic!("No Manager found"), |manager| manager),
        )
        .expect("Failed to create pool")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use r2d2_redis::redis::{Cmd, Commands, ConnectionLike, Value};
    use rstest::{fixture, rstest};
    use std::thread::spawn;

    use crate::settings::{self};

    use super::*;

    #[fixture]
    fn pool() -> Pool<RedisConnectionManager> {
        let manager = RedisConnectionManager::new(settings::get().unwrap().redis.url)
            .expect("Failed to create Redis manager");
        establish_connection(&settings::get().unwrap(), Data::new(manager))
    }

    #[rstest]
    fn test_can_write_to_redis(pool: Pool<RedisConnectionManager>) {
        let mut conn = pool.get().unwrap();

        // Test query
        conn.set::<&str, &str, String>("test", "test").unwrap();
        let result: String = conn.get("test").unwrap();
        assert_eq!(result, "test");

        // Clean up
        conn.del::<&str, i32>("test").unwrap();
    }

    #[rstest]
    fn test_can_write_to_redis_concurrently(pool: Pool<RedisConnectionManager>) {
        let handles = (0..10)
            .map(|_| {
                let pool = pool.clone();
                spawn(move || {
                    let mut conn = pool.get().unwrap();
                    conn.set::<&str, &str, String>("test_1", "test_1").unwrap();

                    let result: String = conn.get("test_1").unwrap();
                    assert_eq!(result, "test_1");
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[rstest]
    fn test_single_writer_redis(pool: Pool<RedisConnectionManager>) {
        // Insert 5 items using a for loop
        let mut conn = pool.get().unwrap();
        conn.set::<&str, &str, String>("test_2", "test_2").unwrap();

        // Verify the count matches
        let result: String = conn.get("test_2").unwrap();
        assert_eq!(result, "test_2");

        // Clean up
        conn.del::<&str, i32>("test_2").unwrap();
    }

    #[rstest]
    fn test_write_and_read_redis(pool: Pool<RedisConnectionManager>) {
        // Create table if not exists
        let mut conn = pool.get().unwrap();

        for i in 0..5 {
            conn.set::<&str, &str, String>(&format!("test_{}", i + 3), &format!("test_{}", i + 3))
                .unwrap();
        }

        for i in 0..5 {
            let result: String = conn.get(format!("test_{}", i + 3)).unwrap();
            assert_eq!(result, format!("test_{}", i + 3));
        }

        // Clean up
        for i in 0..5 {
            assert!(conn.del::<&str, i32>(&format!("test_{}", i + 3)).unwrap() > 0);
        }
    }

    #[rstest]
    fn test_basic_connection_and_ping_redis(pool: Pool<RedisConnectionManager>) {
        let mut conn = pool.get().unwrap();
        // Basic ping command to verify connection
        let result = conn.req_command(Cmd::new().arg("PING")).unwrap();
        assert_eq!(result, Value::Status(String::from("PONG")));
    }

    #[rstest]
    fn test_string_operations_redis(pool: Pool<RedisConnectionManager>) {
        // Establish a connection
        let mut conn = pool.get().unwrap();

        // Set a key-value pair
        let test_key = "test_key";
        let test_value = "Test Value";

        conn.set::<&str, &str, String>(test_key, test_value)
            .unwrap();

        // Retrieve and assert the value
        let retrieved: String = conn.get::<&str, String>(test_key).unwrap();
        assert_eq!(retrieved, test_value);

        // Clean up
        conn.del::<&str, i32>(test_key).unwrap();
    }

    #[rstest]
    fn test_list_operations_redis(pool: Pool<RedisConnectionManager>) {
        // Establish a connection
        let mut conn = pool.get().unwrap();

        // Test list operations
        let test_list = "test_list";

        // Push elements into the list
        conn.rpush::<&str, &str, i32>(test_list, "element1")
            .unwrap();
        conn.rpush::<&str, &str, i32>(test_list, "element2")
            .unwrap();
        conn.rpush::<&str, &str, i32>(test_list, "element3")
            .unwrap();

        // Retrieve all elements and assert count and values
        let elements: Vec<String> = conn.lrange(test_list, 0, -1).unwrap();

        // Remove the list
        for _ in 0..elements.len() {
            conn.rpop::<&str, String>(test_list).unwrap();
        }
        assert_eq!(elements.len(), 3);
        assert_eq!(
            elements,
            vec![
                "element1".to_string(),
                "element2".to_string(),
                "element3".to_string()
            ]
        );

        // Clean up
        conn.del::<&str, i32>(test_list).unwrap();
    }

    #[rstest]
    fn test_key_expiration_redis(pool: Pool<RedisConnectionManager>) {
        // Establish a connection
        let mut conn = pool.get().unwrap();

        // Test key expiration
        let test_key = "test_expired_key";

        // Set key with expiration in 1 second
        conn.set_ex::<&str, &str, String>(test_key, "Value", 1)
            .unwrap();

        // Wait for expiration
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Try to get the key after expiration
        let retrieved: Option<String> = conn.get(test_key).unwrap();

        assert!(
            retrieved.is_none(),
            "Key should have expired and been removed"
        );
    }
}
