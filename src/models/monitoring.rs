// Model layer: domain structs plus small display helpers used by services and templates.

use super::{formatting::title_case_code, Money};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
// Domain record used by services, repositories and templates.
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
    // Formats the value for display in templates.
    pub fn amount_display(&self) -> String {
        Money::from_cents(self.amount_cents).display()
    }

    // Formats the value for display in templates.
    pub fn rule_display(&self) -> String {
        match self.rule_code.as_str() {
            "HIGH_VALUE_MONITORING" => "High-Value Transaction".to_string(),
            "HIGH_VALUE_REVIEW" => "High-Value Review Hold".to_string(),
            other => title_case_code(other),
        }
    }

    // Formats the value for display in templates.
    pub fn severity_display(&self) -> String {
        title_case_code(&self.severity)
    }

    // Formats the value for display in templates.
    pub fn status_display(&self) -> String {
        match self.status.as_str() {
            "blocked" => "Blocked".to_string(),
            "flagged" | "reviewed" => "Flagged".to_string(),
            "cleared" => "Cleared".to_string(),
            other => title_case_code(other),
        }
    }

    // Formats the value for display in templates.
    pub fn account_display(&self) -> String {
        self.account_number
            .clone()
            .unwrap_or_else(|| "Account not captured".to_string())
    }

    // Formats the value for display in templates.
    pub fn product_display(&self) -> String {
        self.product_id_code
            .as_deref()
            .map(title_case_code)
            .unwrap_or_else(|| "Unknown product".to_string())
    }

    // Formats the value for display in templates.
    pub fn review_notes_display(&self) -> String {
        self.review_notes
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("No admin notes recorded yet.")
            .to_string()
    }

    // Formats the value for display in templates.
    pub fn reviewed_at_display(&self) -> String {
        self.reviewed_at
            .map(|value| value.format("%d %b %Y, %I:%M %p").to_string())
            .unwrap_or_else(|| "Not cleared yet".to_string())
    }

    // Formats the value for display in templates.
    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y, %I:%M %p").to_string()
    }

    // Returns whether this record is blocked.
    pub fn is_blocked(&self) -> bool {
        self.status == "blocked"
    }

    // Returns whether this record is flagged.
    pub fn is_flagged(&self) -> bool {
        self.status == "flagged" || self.status == "reviewed"
    }

    // Returns whether this record is cleared.
    pub fn is_cleared(&self) -> bool {
        self.status == "cleared"
    }
}
