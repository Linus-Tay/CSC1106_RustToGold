use chrono::{NaiveDate, NaiveDateTime};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct StaffUser {
    pub id: i64,
    pub full_name: String,
    pub email: String,
    pub phone_number: String,
    pub date_of_birth: NaiveDate,
    pub password_hash: String,
    pub role: String,
    pub status: String,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl StaffUser {
    pub fn status_display(&self) -> String {
        match self.status.as_str() {
            "active" => "Active".to_string(),
            "suspended" => "Suspended".to_string(),
            "closed" => "Closed".to_string(),
            other => other.to_string(),
        }
    }

    pub fn role_display(&self) -> String {
        match self.role.as_str() {
            "staff" => "Staff".to_string(),
            "admin" => "Admin".to_string(),
            "customer" => "Customer".to_string(),
            other => other.to_string(),
        }
    }

    pub fn date_of_birth_display(&self) -> String {
        self.date_of_birth.format("%d %b %Y").to_string()
    }

    pub fn last_login_display(&self) -> String {
        match self.last_login_at {
            Some(dt) => dt.format("%d %b %Y %H:%M").to_string(),
            None => "Never".to_string(),
        }
    }

    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y").to_string()
    }

    pub fn is_active(&self) -> bool {
        self.status == "active"
    }

    pub fn is_staff(&self) -> bool {
        self.role == "staff"
    }
}
