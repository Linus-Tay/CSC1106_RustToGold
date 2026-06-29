use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::{Serialize, Deserialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AccountCreationLink {
    id: Uuid,
    customer_id: Uuid,
    status: String,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl AccountCreationLink {
    pub fn is_valid(&self) -> bool {
         self.status == "pending".to_string() && self.expires_at > Utc::now()
    }

    pub fn get_link(&self) -> String {
        self.id.to_string()
    }

    pub fn get_customer_id(&self) -> Uuid {
        self.customer_id
    }
}
