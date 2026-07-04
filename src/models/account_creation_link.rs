use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AccountCreationLink {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl AccountCreationLink {
    // Returns whether this record is valid
    pub fn is_valid(&self) -> bool {
        self.status == "pending" && self.expires_at > Utc::now()
    }

    // Returns the link field
    pub fn get_link(&self) -> String {
        self.id.to_string()
    }

    // Returns the customer id field
    pub fn get_customer_id(&self) -> Uuid {
        self.customer_id
    }
}
