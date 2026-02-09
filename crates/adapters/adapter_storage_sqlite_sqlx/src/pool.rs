//! `SQLite` connection pool setup and migration runner.

use std::str::FromStr;

use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;

use crate::error::StorageError;

/// Configuration for the `SQLite` storage adapter.
pub struct Config {
    /// `SQLite` connection URL (e.g. `sqlite:minihub.db` or `sqlite::memory:`).
    pub database_url: String,
}

impl Config {
    /// Read configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if `MINIHUB_DATABASE_URL` is not set.
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            database_url: std::env::var("MINIHUB_DATABASE_URL")?,
        })
    }

    /// Build a [`Database`] from this configuration.
    ///
    /// Creates the connection pool, creates the database file if missing,
    /// and runs all pending migrations.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError`] if the connection or migrations fail.
    pub async fn build(self) -> Result<Database, StorageError> {
        Database::initialize(&self.database_url).await
    }
}

/// Holds the `SQLite` connection pool and provides access to it.
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Connect to the database and run migrations.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError`] if the connection or migrations fail.
    async fn initialize(database_url: &str) -> Result<Self, StorageError> {
        let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    /// Borrow the underlying connection pool.
    #[must_use]
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_create_pool_and_run_migrations_when_using_memory_db() {
        let config = Config {
            database_url: "sqlite::memory:".to_string(),
        };
        let db = config.build().await.unwrap();

        // Verify tables exist by querying sqlite_master
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations' ORDER BY name",
        )
        .fetch_all(db.pool())
        .await
        .unwrap();

        let names: Vec<&str> = tables.iter().map(|row| row.0.as_str()).collect();
        assert!(names.contains(&"areas"), "missing areas table");
        assert!(names.contains(&"devices"), "missing devices table");
        assert!(names.contains(&"entities"), "missing entities table");
        assert!(names.contains(&"events"), "missing events table");
    }
}
