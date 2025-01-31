use crate::endpoints::{health, index, templates};
use crate::settings::Settings;
use actix_web::{http::KeepAlive, middleware, App, HttpServer};
use std::net;
use tracing::{debug, info, instrument, warn};

pub const PARSE_COUNT: u8 = 9;

#[instrument(
    name = "Running the server",
    target = "demo_web_app",
    level = "info",
    skip(listener, settings)
)]
fn run(
    listener: std::net::TcpListener,
    settings: Settings,
) -> Result<actix_web::dev::Server, std::io::Error> {
    // let manager = r2d2_sqlite::SqliteConnectionManager::file(settings.sqlite.path.clone());
    // let manager = r2d2_sqlite::SqliteConnectionManager::memory();

    // let pool = r2d2::Pool::new(manager).expect("Failed to create the connection pool");

    // Connect to the MongoDB database
    // let db_data = Data::new(pool);
    // info!("Processed DB connection pool for distribution");

    // Redis connection pool

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            // .wrap(middleware::DefaultHeaders::new().add(("X-Version", env!("CARGO_PKG_VERSION"))))
            // .app_data(db_data.clone())
            .service(templates::favicon)
            .service(templates::logomain)
            .service(templates::stylesheet)
            .service(templates::source_map)
            .service(templates::htmx)
            .service(templates::hyperscript)
            .service(templates::response_targets)
            .service(templates::sse)
            .service(templates::action_script)
            .service(templates::gif)
            .service(index::index)
            .service(health::health_check)
    })
    .keep_alive(KeepAlive::Os) // Keep the connection alive; OS handled
    .disable_signals() // Disable the signals to allow the OS to handle the signals
    .workers(2)
    .shutdown_timeout(3)
    .listen(listener)?
    .run();

    if settings.debug {
        warn!("Debug mode");
    } else {
        warn!("Production mode");
    }

    Ok(server)
}

pub struct Application {
    port: u16,
    server: actix_web::dev::Server,
}

impl Application {
    /// # Result
    ///  - `Ok(Application)` if the application was successfully built
    /// # Errors
    ///  - `std::io::Error` if the application could not be built
    /// # Panics
    ///  - If the application could not be built
    #[instrument(
        name = "Build Application",
        level = "info",
        target = "demo_web_app",
        skip(settings)
    )]
    pub async fn build(settings: &mut crate::settings::Settings) -> Result<Self, std::io::Error> {
        info!("Buidling the main application");
        // let connection_pool = if let Some(pool) = test_pool {
        //     pool
        // } else {
        //     get_connection_pool(&settings.mongo).await
        // };

        // info!("Init or touch the DB");
        // let _connection_pool = match create_schema(&mut settings.sqlite) {
        //     Ok(conn) => conn,
        //     Err(err) => {
        //         error!("Failed to create the SQLite database: {err}\nExiting...");
        //         panic!("Failed to create the SQLite database");
        //     }
        // };

        let address = format!(
            "{}:{}",
            settings.application.host, settings.application.port
        );

        debug!("Binding the TCP port: {address}");
        let listener: net::TcpListener = net::TcpListener::bind(&address)?;
        let port = listener.local_addr()?.port();
        let server = run(listener, settings.clone())?;

        Ok(Self { port, server })
    }

    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// # Result
    ///  - `Ok(())` if the application was successfully started
    /// # Errors
    ///  - `std::io::Error` if the application could not be started
    /// # Panics
    ///  - If the application could not be started
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        info!("Running until stopped");
        self.server.await
    }
}

/*
/// # Result
///  - `Ok(Database)` if the connection pool was successfully created
/// # Errors
///  - `mongodb::error::Error` if the connection pool could not be created
/// # Panics
///  - If the connection pool could not be created
#[instrument(name = "Get Connection Pool", level = "info", target = "demo_web_app")]
async fn get_connection_pool(settings: &settings::Mongo) -> mongodb::Database {
    info!("Get mongo connection pool");
    let mut client_options = settings.mongo_options().await;
    client_options.app_name = Some(settings.clone().db);

    let client = match mongodb::Client::with_options(client_options) {
        Ok(client) => client,
        Err(err) => {
            error!("Failed to connect to MongoDB: {err}\nExiting...");
            panic!("Failed to connect to MongoDB");
        }
    };
    client.database(&settings.db)
}

/// # Result
/// - `Ok(())` Created the sqlite3 database file if it did not exist
/// # Errors
/// - `std::io::Error` if the database file could not be created
/// # Panics
/// - If the database file could not be created
#[instrument(name = "Create Database", level = "info", target = "demo_web_app")]
pub fn create_schema(
    settings: &mut settings::Sqlite,
) -> Result<rusqlite::Connection, rusqlite::Error> {
    info!("Creating the SQLite database schema");

    let conn = match File::create(&settings.path) {
        Ok(_) => {
            info!("Successfully created the SQLite database file");
            Connection::open(&settings.path)
                .map_err(|err| {
                    error!("Failed to open the SQLite database: {err}\nExiting...");
                })
                .expect("Failed to open the SQLite database")
        }
        Err(err) => {
            error!("Failed to create the SQLite database file: {err}\nExiting...");
            panic!("Failed to create the SQLite database file");
        }
    };

    let schema = settings.schema.clone();

    for line in schema.lines() {
        warn!("Schema line: {line}");
    }

    let schema = fs::read_to_string(&settings.schema).expect("Failed to read the schema file");

    match conn.execute_batch(&schema) {
        Ok(()) => {
            info!("Successfully created the SQLite database schema");
        }
        Err(err) => {
            error!("Failed to create the SQLite database schema: {err}\nExiting...");
        }
    }

    Ok(conn)
}
*/
