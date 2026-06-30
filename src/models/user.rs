use chrono::{NaiveDate, NaiveDateTime};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub customer_id: Uuid,
    #[sqlx(default)]
    pub full_name: Option<String>,
    #[sqlx(default)]
    pub date_of_birth: Option<NaiveDate>,
    #[sqlx(default)]
    pub phone_number: Option<String>,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub status: String,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl User {
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }

    pub fn is_customer(&self) -> bool {
        self.role == "customer"
    }

    pub fn joined_display(&self) -> String {
        self.created_at.format("%d %b %Y").to_string()
    }

    pub fn last_login_display(&self) -> String {
        self.last_login_at
            .map(|value| value.format("%d %b %Y, %I:%M %p").to_string())
            .unwrap_or_else(|| "Not recorded yet".to_string())
    }
}
