use super::{formatting::title_case_code, Money};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct HighValueAlertRecord {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub customer_name: String,
    pub customer_email: String,
    pub product_id: Option<Uuid>,
    pub account_number: Option<String>,
    pub product_id_code: Option<String>,
    pub rule_code: String,
    pub severity: String,
    pub channel: String,
    pub amount_cents: i64,
    pub message: String,
    pub status: String,
    pub review_notes: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl HighValueAlertRecord {
    pub fn amount_display(&self) -> String {
        Money::from_cents(self.amount_cents).display()
    }

    pub fn rule_display(&self) -> String {
        match self.rule_code.as_str() {
            "HIGH_VALUE_MONITORING" => "High-Value Transaction".to_string(),
            "HIGH_VALUE_REVIEW" => "High-Value Review Hold".to_string(),
            other => title_case_code(other),
        }
    }

    pub fn severity_display(&self) -> String {
        title_case_code(&self.severity)
    }

    pub fn status_display(&self) -> String {
        match self.status.as_str() {
            "blocked" => "Blocked".to_string(),
            "flagged" | "reviewed" => "Flagged".to_string(),
            "cleared" => "Cleared".to_string(),
            other => title_case_code(other),
        }
    }

    pub fn account_display(&self) -> String {
        self.account_number
            .clone()
            .unwrap_or_else(|| "Account not captured".to_string())
    }

    pub fn product_display(&self) -> String {
        self.product_id_code
            .as_deref()
            .map(title_case_code)
            .unwrap_or_else(|| "Unknown product".to_string())
    }

    pub fn review_notes_display(&self) -> String {
        self.review_notes
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("No admin notes recorded yet.")
            .to_string()
    }

    pub fn reviewed_at_display(&self) -> String {
        self.reviewed_at
            .map(|value| value.format("%d %b %Y, %I:%M %p").to_string())
            .unwrap_or_else(|| "Not cleared yet".to_string())
    }

    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y, %I:%M %p").to_string()
    }

    pub fn is_blocked(&self) -> bool {
        self.status == "blocked"
    }

    pub fn is_flagged(&self) -> bool {
        self.status == "flagged" || self.status == "reviewed"
    }

    pub fn is_cleared(&self) -> bool {
        self.status == "cleared"
    }
}
