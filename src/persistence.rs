use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

use crate::config::Config;

async fn pool(cfg: &Config) -> sqlx::Result<SqlitePool> {
    SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await
}
