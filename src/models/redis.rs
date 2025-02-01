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
        .connection_timeout(Duration::from_secs(settings.redis.connection_timeout))
        .build(
            Arc::into_inner(manager.into_inner())
                .map_or_else(|| panic!("No Manager found"), |manager| manager),
        )
        .expect("Failed to create pool")
}
