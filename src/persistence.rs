use chrono::NaiveDateTime;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tokio::sync::broadcast;
use tracing::info;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct Notification {
    pub app_name: String,
    pub summary: String,
    pub body: String,
    /// In UTC.
    pub created_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct Persistence {
    db: SqlitePool,
}

impl Persistence {
    pub async fn new(cfg: &Config) -> sqlx::Result<Self> {
        let db = Self::pool(cfg).await?;
        Ok(Self { db })
    }

    /// Receives notifications from `rx` and persist them in the database.
    #[tracing::instrument(skip_all)]
    pub async fn persist(
        mut self,
        mut rx: broadcast::Receiver<crate::notification::Notification>,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) -> sqlx::Result<()> {
        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    info!("Shutdown signal received, exiting task...");
                    return Ok(());
                }
                Ok(notification) = rx.recv() => {
                    self.insert_notification(notification).await?;
                }
                else => break,
            }
        }

        Ok(())
    }

    async fn insert_notification(
        &mut self,
        notification: crate::notification::Notification,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO notifications
                    (summary, body, app_name)
             VALUES (?, ?, ?)",
            notification.app_name,
            notification.summary,
            notification.body,
        )
        .execute(&self.db)
        .await
        .map(drop)
    }

    async fn pool(cfg: &Config) -> sqlx::Result<SqlitePool> {
        SqlitePoolOptions::new()
            .max_connections(10)
            .connect(&cfg.database_url)
            .await
    }
}
