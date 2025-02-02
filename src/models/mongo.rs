//! Initialize and return a connection to the ``MongoDb`` database.

use super::r2d2_mongodb::client_manager::MongoClientManager;
use crate::settings;
use actix_web::web::Data;
use mongodb::{bson::Document, Collection};
use r2d2::ManageConnection;
use std::sync::Arc;
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
) -> Collection<Document> {
    info!("Get mongo connection pool");
    Arc::into_inner(
        manager
            .into_inner()
            .connect()
            .expect("No Mongodb Manager found")
            .database(&settings.db)
            .collection(&settings.collection)
            .into(),
    )
    .expect("Failed to create pool")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::time::Duration;

    use mongodb::{
        bson::{doc, Bson, Document},
        options::{ClientOptions, ServerAddress},
        Collection,
    };
    use r2d2::ManageConnection;
    use rstest::rstest;

    use crate::settings::{self, Settings};

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn test_can_establish_connection() {
        let settings: Settings = settings::get().unwrap();

        let manager = MongoClientManager::new(
            ClientOptions::builder()
                .app_name(settings.mongo.db.clone())
                .hosts(vec![ServerAddress::Tcp {
                    host: settings.mongo.host.clone(),
                    port: Some(settings.mongo.port),
                }])
                .max_pool_size(Some(settings.mongo.pool_size.into()))
                .connect_timeout(Duration::from_secs(
                    settings.mongo.connection_timeout.into(),
                ))
                .build(),
        );

        let pool = establish_connection(&settings::get().unwrap().mongo, Data::new(manager)).await;

        // Assert that a connection has been established
        assert!(pool.estimated_document_count().await.is_ok());
    }

    #[rstest]
    #[tokio::test]
    async fn test_fail_to_connect() {
        let manager = MongoClientManager::new(
            ClientOptions::builder()
                .app_name(Some(String::from("testing")))
                .hosts(vec![ServerAddress::Tcp {
                    host: "localhost".to_string(),
                    port: Some(27017),
                }])
                .max_pool_size(Some(10))
                .connect_timeout(Duration::from_secs(1))
                .server_selection_timeout(Duration::from_secs(1))
                .build(),
        );

        let pool = establish_connection(&settings::get().unwrap().mongo, Data::new(manager)).await;

        // Assert that a connection has been established
        assert!(pool.estimated_document_count().await.is_err());
    }

    #[rstest]
    #[tokio::test]
    async fn test_can_write_to_mongo() {
        let settings: Settings = settings::get().unwrap();

        let conn = MongoClientManager::from_uri(&settings.mongo.uri)
            .await
            .unwrap()
            .connect()
            .unwrap();

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
        let db = conn.database("test_1");
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

    #[rstest]
    #[tokio::test]
    async fn test_can_update_mongo() {
        let settings: Settings = settings::get().unwrap();

        let conn = MongoClientManager::from_uri(&settings.mongo.uri)
            .await
            .unwrap()
            .connect()
            .unwrap();

        // Test query the document database, mongodb
        let db = conn.database("test_2");
        let collection = db.collection("test_2");

        // Test query
        let _ = collection.insert_one(doc! { "name": "John Dae" }).await;

        // Test query
        let result = collection
            .update_one(
                doc! { "name": "John Dae" },
                doc! { "$set": { "name": "John OtherDoe" } },
            )
            .await
            .unwrap();

        let changed_result = collection
            .find_one(doc! { "name": "John OtherDoe" })
            .await
            .unwrap();

        // Drop the database
        assert!(db.drop().await.is_ok());

        assert!(result.modified_count.eq(&1));

        assert!(changed_result
            .unwrap()
            .get_str("name")
            .unwrap()
            .eq("John OtherDoe"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_not_found_mongo() {
        let settings: Settings = settings::get().unwrap();

        let conn = MongoClientManager::from_uri(&settings.mongo.uri)
            .await
            .unwrap()
            .connect()
            .unwrap();

        // Test query the document database, mongodb
        let db = conn.database("test_3");
        let collection: Collection<Document> = db.collection("test_3");

        // insert a document
        let _ = collection
            .insert_one(doc! { "house": "180 SW 125th Ave" })
            .await;

        // Test query
        let result = collection
            .find_one(doc! { "name": "Jane Doe" })
            .await
            .unwrap();

        // Drop the database
        assert!(db.drop().await.is_ok());

        assert!(result.is_none());
    }
}
