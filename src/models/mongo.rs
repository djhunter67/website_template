//! Initialize and return a connection to the ``MongoDb`` database.

use super::r2d2_mongodb::client_manager::MongoClientManager;
use crate::settings;
use actix_web::web::Data;
use r2d2::Pool;
use std::{sync::Arc, time::Duration};
use tracing::{info, instrument};

#[must_use]
#[instrument(
    name = "Get Connection Pool for MongoDb",
    level = "info",
    target = "demo_web_app",
    skip(settings, manager)
)]
/// # Result
///  - `Ok(Database)` if the connection pool was successfully created
/// # Errors
///  - `mongodb::error::Error` if the connection pool could not be created
/// # Panics
///  - If the connection pool could not be created
pub async fn establish_connection(
    settings: &settings::Mongo,
    manager: Data<MongoClientManager>,
) -> Pool<MongoClientManager> {
    info!("Get mongo connection pool");
    Pool::builder()
        .max_size(settings.pool_size.into())
        .connection_timeout(Duration::from_secs(settings.connection_timeout.into()))
        .build(Arc::into_inner(manager.into_inner()).expect("No Mongodb Manager found"))
        .expect("Failed to create pool")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use mongodb::bson::{doc, Bson};
    use r2d2::ManageConnection;
    use rstest::rstest;

    use crate::settings::{self, Settings};

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn test_can_write_to_mongo() {
        let settings: Settings = settings::get().unwrap();

        let conn = MongoClientManager::from_uri(&settings.mongo.uri)
            .await
            .unwrap()
            .connect()
            .unwrap();
        // let pool = establish_connection(&settings.mongo, Data::new(manager)).await;

        // let conn = pool.get().unwrap();

        // Test query the document database, mongodb
        let db = conn.database("test");
        let collection = db.collection("test");

        // Test query
        let result = collection
            .insert_one(doc! { "name": "John Doe" })
            .await
            .unwrap();

        // Drop the database
        assert!(db.drop().await.is_ok());

        assert!(result.inserted_id.ne(&Bson::Null));
    }

    #[rstest]
    #[tokio::test]
    async fn test_can_read_from_mongo() {
        let settings: Settings = settings::get().unwrap();

        let conn = MongoClientManager::from_uri(&settings.mongo.uri)
            .await
            .unwrap()
            .connect()
            .unwrap();

        // Test query the document database, mongodb
        let db = conn.database("test");
        let collection = db.collection("test_1");

        // Test query
        let _ = collection.insert_one(doc! { "name": "John Dae" }).await;

        // Test query
        let result = collection
            .find_one(doc! { "name": "John Dae" })
            .await
            .unwrap();

        // Drop the database
        assert!(db.drop().await.is_ok());

        assert!(result.unwrap().get_str("name").unwrap().eq("John Dae"));
    }
}
