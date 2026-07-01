use chrono::NaiveDateTime;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub customer_id: Option<Uuid>,
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
        self.role == "customer" && self.customer_id.is_some()
    }

    pub fn is_staff_or_admin(&self) -> bool {
        matches!(self.role.as_str(), "staff" | "admin")
    }

    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    pub fn customer_id_or_nil(&self) -> Uuid {
        self.customer_id.unwrap_or_else(Uuid::nil)
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
