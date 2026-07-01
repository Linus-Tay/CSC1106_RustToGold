// Model layer: domain structs plus small display helpers used by services and templates.

use super::Money;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
// Domain record used by services, repositories and templates.
pub struct PersonalLoan {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub funding_product_id: Uuid,
    pub purpose: String,
    pub principal_cents: i64,
    pub annual_rate_bps: i32,
    pub term_months: i32,
    pub monthly_payment_cents: i64,
    pub outstanding_cents: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PersonalLoan {
    // Formats the value for display in templates.
    pub fn principal_display(&self) -> String {
        Money::from_cents(self.principal_cents).display()
    }

    // Formats the value for display in templates.
    pub fn monthly_payment_display(&self) -> String {
        Money::from_cents(self.monthly_payment_cents).display()
    }

    // Formats the value for display in templates.
    pub fn outstanding_display(&self) -> String {
        Money::from_cents(self.outstanding_cents).display()
    }

    // Returns the raw value used by form fields and validation.
    pub fn outstanding_plain(&self) -> String {
        format!("{:.2}", self.outstanding_cents as f64 / 100.0)
    }

    // Formats the value for display in templates.
    pub fn rate_display(&self) -> String {
        format!("{:.2}%", self.annual_rate_bps as f64 / 100.0)
    }

    // Formats the value for display in templates.
    pub fn status_display(&self) -> String {
        match self.status.as_str() {
            "pending" => "Pending Review".to_string(),
            "active" => "Active".to_string(),
            "rejected" => "Rejected".to_string(),
            "fully_paid" => "Fully Paid".to_string(),
            "cancelled" => "Cancelled".to_string(),
            value => value.replace('_', " "),
        }
    }

    // Formats the value for display in templates.
    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y").to_string()
    }

    // Returns whether this record is payable.
    pub fn is_payable(&self) -> bool {
        self.status == "active" && self.outstanding_cents > 0
    }

    // Explains the approval state beside each customer loan record.
    pub fn customer_status_note(&self) -> &'static str {
        match self.status.as_str() {
            "pending" => "Awaiting bank review. No funds have been disbursed yet.",
            "active" => "Approved and disbursed into your selected account.",
            "rejected" => "Application was not approved.",
            "fully_paid" => "Loan has been fully repaid.",
            _ => "Status updated by the bank.",
        }
    }
}
