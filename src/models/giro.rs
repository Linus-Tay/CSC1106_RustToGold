// Model layer: domain structs plus small display helpers used by services and templates.

use super::{formatting::title_case_code, Money};
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
// Domain record used by services, repositories and templates.
pub struct GiroArrangement {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub from_product_id: Uuid,
    pub account_number: String,
    pub recipient_product_id: Uuid,
    pub recipient_account_number: String,
    pub recipient_account_label: String,
    pub payee_name: String,
    pub amount_cents: i64,
    pub frequency: String,
    pub next_payment_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub note: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GiroArrangement {
    // Formats the value for display in templates.
    pub fn id_display(&self) -> String {
        self.id.to_string()
    }

    // Formats the value for display in templates.
    pub fn amount_display(&self) -> String {
        Money::from_cents(self.amount_cents).display()
    }

    // Formats the value for display in templates.
    pub fn frequency_display(&self) -> String {
        title_case_code(&self.frequency)
    }

    // Formats the value for display in templates.
    pub fn status_display(&self) -> String {
        title_case_code(&self.status)
    }

    // Formats the value for display in templates.
    pub fn next_payment_date_display(&self) -> String {
        self.next_payment_date.format("%d %b %Y").to_string()
    }

    // Formats the value for display in templates.
    pub fn end_date_display(&self) -> String {
        self.end_date
            .map(|date| date.format("%d %b %Y").to_string())
            .unwrap_or_else(|| "Until cancelled".to_string())
    }

    // Formats the value for display in templates.
    pub fn note_display(&self) -> &str {
        self.note.as_deref().unwrap_or("No reference")
    }

    // Returns whether this record is active.
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }
}
