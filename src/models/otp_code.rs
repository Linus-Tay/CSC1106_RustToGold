use chrono::{DateTime, Utc, Duration};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
// Domain record used by services, repositories and templates.
pub struct OTPCode {
    pub id: Uuid,
    pub code: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl OTPCode {
    // Returns whether this device is active.
    pub fn is_active(&self) -> bool {
        self.expires_at > Utc::now()
    }
}
