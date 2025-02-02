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
pub fn establish_connection(
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
#[allow(clippy::unwrap_used, clippy::future_not_send)]
mod tests {

    use mongodb::{
        bson::{doc, Bson, Document},
        Collection,
    };
    use r2d2::ManageConnection;
    use rstest::{fixture, rstest};

    use crate::settings::{self, Mongo, Settings};

    use super::*;

    #[fixture]
    async fn manager() -> Collection<Document> {
        let settings: Settings = settings::get().unwrap();
        let manager = MongoClientManager::from_uri(&settings.mongo.uri)
            .await
            .unwrap();

        let mongo = Mongo {
            username: "root".to_string(),
            password: "password".to_string(),
            require_auth: false,
            db: "test".to_string(),
            collection: "test".to_string(),
            uri: settings.mongo.uri,
            pool_size: 16,
            connection_timeout: 5,
        };

        establish_connection(&mongo, Data::new(manager))
    }

    #[rstest]
    #[tokio::test]
    async fn test_can_write_to_mongo(#[future] manager: Collection<Document>) {
        let collection = manager.await;

        // Test query
        let result = collection
            .insert_one(doc! { "name": "John Doe" })
            .await
            .unwrap();

        assert!(result.inserted_id.ne(&Bson::Null));

        // test_cleanup(&settings::get().unwrap()).await;
    }

    #[rstest]
    #[tokio::test]
    async fn test_can_read_from_mongo(#[future] manager: Collection<Document>) {
        let collection = manager.await;

        // Test query
        let _ = collection.insert_one(doc! { "name": "John Dae" }).await;

        // Test query
        let result = collection
            .find_one(doc! { "name": "John Dae" })
            .await
            .unwrap();

        assert!(result.unwrap().get_str("name").unwrap().eq("John Dae"));

        // test_cleanup(&settings::get().unwrap()).await;
    }

    #[rstest]
    #[tokio::test]
    async fn test_can_update_mongo(#[future] manager: Collection<Document>) {
        let collection = manager.await;

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

        assert!(result.modified_count.eq(&1));

        assert!(changed_result
            .unwrap()
            .get_str("name")
            .unwrap()
            .eq("John OtherDoe"));

        // test_cleanup(&settings::get().unwrap()).await;
    }

    #[rstest]
    #[tokio::test]
    async fn test_not_found_mongo(#[future] manager: Collection<Document>) {
        let collection: Collection<Document> = manager.await;

        // insert a document
        let _ = collection
            .insert_one(doc! { "house": "180 SW 125th Ave" })
            .await;

        // Test query
        let result = collection
            .find_one(doc! { "name": "Jane Doe" })
            .await
            .unwrap();

        assert!(result.is_none());
        test_cleanup(&settings::get().unwrap()).await;
    }

    async fn test_cleanup(settings: &Settings) {
        let manager = MongoClientManager::from_uri(&settings.mongo.uri)
            .await
            .unwrap()
            .connect()
            .unwrap()
            .database("test");

        // Drop the database
        manager.drop().await.unwrap();
    }
}
