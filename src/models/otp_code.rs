use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct OTPCode {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl OTPCode {
    // Returns whether this device is active.
    pub fn is_active(&self) -> bool {
        self.expires_at > Utc::now()
    }

    // Returns the user id
    pub fn get_user_id(&self) -> Uuid {
        self.user_id
    }
}
