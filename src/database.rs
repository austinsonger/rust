use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool, PooledConnection, PoolError},
    Connection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use once_cell::sync::Lazy;
use std::{error::Error, time::Duration};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::{
    constants::database::{CONNECTION_RETRY_DELAY_MS, MAX_CONNECTION_ATTEMPTS},
    errors,
    settings::SETTINGS,
};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

// Embed migrations
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

// Create a database pool as a global static
static DB_POOL: Lazy<DbPool> = Lazy::new(|| {
    create_db_pool(&SETTINGS.database.url, SETTINGS.database.pool_size)
        .expect("Failed to create database pool")
});

/// Create a new database connection pool
fn create_db_pool(database_url: &str, pool_size: u32) -> Result<DbPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(pool_size)
        .connection_timeout(Duration::from_secs(30))
        .build(manager)
}

/// Get a connection from the pool
pub fn get_connection() -> Result<DbConnection, errors::Error> {
    let mut attempts = 0;
    let max_attempts = MAX_CONNECTION_ATTEMPTS;

    loop {
        attempts += 1;
        match DB_POOL.get() {
            Ok(conn) => {
                debug!("Database connection acquired");
                return Ok(conn);
            }
            Err(err) => {
                if attempts >= max_attempts {
                    error!(
                        error = %err,
                        "Failed to get database connection after {max_attempts} attempts"
                    );
                    return Err(errors::Error::database_error(
                        format!("Failed to get database connection: {}", err),
                        Some(Box::new(err)),
                        None,
                    ));
                }

                warn!(
                    attempt = attempts,
                    max_attempts = max_attempts,
                    "Failed to get database connection, retrying..."
                );

                // Sleep for a short time before retrying
                std::thread::sleep(Duration::from_millis(CONNECTION_RETRY_DELAY_MS));
            }
        }
    }
}

/// Run database migrations
pub fn run_migrations() -> Result<(), Box<dyn Error>> {
    info!("Running database migrations...");

    // Get a connection from the pool
    let mut conn = PgConnection::establish(&SETTINGS.database.url)
        .map_err(|e| {
            error!(error = %e, "Failed to establish database connection for migrations");
            Box::new(e) as Box<dyn Error>
        })?;

    // Run the migrations
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| {
            error!(error = %e, "Failed to run database migrations");
            Box::new(e) as Box<dyn Error>
        })?;

    info!("Database migrations completed successfully");
    Ok(())
}

/// Check if the database is available
pub async fn check_database_availability() -> bool {
    let mut attempts = 0;
    let max_attempts = MAX_CONNECTION_ATTEMPTS;

    while attempts < max_attempts {
        attempts += 1;

        match get_connection() {
            Ok(_) => {
                info!("Database is available");
                return true;
            }
            Err(err) => {
                warn!(
                    error = %err,
                    attempt = attempts,
                    max_attempts = max_attempts,
                    "Database is not available, retrying..."
                );

                // Sleep for a short time before retrying
                sleep(Duration::from_millis(CONNECTION_RETRY_DELAY_MS)).await;
            }
        }
    }

    error!("Database is not available after {max_attempts} attempts");
    false
}
