use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use once_cell::sync::Lazy;
use std::error::Error;
use std::time::Duration;

use crate::settings::SETTINGS;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

static CONNECTION_POOL: Lazy<DbPool> = Lazy::new(|| {
    let database_url = SETTINGS.database.url.clone();
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder()
        .connection_timeout(Duration::from_secs(30))
        .max_size(SETTINGS.database.pool_size)
        .build(manager)
        .expect("Failed to create database connection pool")
});

pub fn get_connection() -> Result<DbConnection, r2d2::Error> {
    CONNECTION_POOL.get()
}

pub fn run_migrations() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_connection()?;
    conn.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}
