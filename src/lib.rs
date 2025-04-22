use chrono::NaiveDateTime;

pub mod config;
pub mod notification;
pub mod persistence;
pub mod ui;

#[derive(Debug, Clone)]
pub struct Notification {
    pub app_name: String,
    pub summary: String,
    pub body: String,
    /// A URI that was validated by [`Url::parse`](url::Url::parse).
    pub app_icon: Option<String>,
    /// In [`UTC`](chrono::Utc).
    pub created_at: NaiveDateTime,
}
