use chrono::NaiveDateTime;

pub mod notification;
pub mod persistence;
pub mod ui;

#[derive(Debug, Clone)]
pub struct Notification {
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub created_at: NaiveDateTime,
}
