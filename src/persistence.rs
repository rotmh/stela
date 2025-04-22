use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tokio::sync::broadcast;
use tracing::info;

use crate::{Notification, config::Config};

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
    #[tracing::instrument]
    pub async fn persist(
        mut self,
        mut rx: broadcast::Receiver<Notification>,
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
        notification: Notification,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO notifications
                    (summary, body, app_name, app_icon, created_at)
             VALUES (?, ?, ?, ?, ?)",
            notification.app_name,
            notification.summary,
            notification.body,
            notification.app_icon,
            notification.created_at,
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
