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
    // Check is active
    pub fn is_active(&self) -> bool {
        self.expires_at > Utc::now()
    }

    // Return get user id
    pub fn get_user_id(&self) -> Uuid {
        self.user_id
    }
}
