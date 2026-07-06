use chrono::{DateTime, Utc, Duration};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct KnownDevice {
    pub id: Uuid,
    pub token_hash: String,
    pub user_id: Uuid,
    pub last_used: DateTime<Utc>,
}

impl KnownDevice {
    // Check is active
    pub fn is_active(&self) -> bool {
        let thirty_days_ago = Utc::now() - Duration::days(30);
        self.last_used >= thirty_days_ago
    }
}
