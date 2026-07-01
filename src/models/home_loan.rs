// Model layer: domain structs plus small display helpers used by services and templates.

use super::Money;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
// Domain record used by services, repositories and templates.
pub struct HomeLoanApplication {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub account_product_id: Option<Uuid>,
    pub property_type: String,
    pub property_value_cents: i64,
    pub down_payment_cents: i64,
    pub loan_amount_cents: i64,
    pub annual_rate_bps: i32,
    pub term_years: i32,
    pub monthly_payment_cents: i64,
    pub outstanding_cents: i64,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl HomeLoanApplication {
    // Formats the value for display in templates.
    pub fn property_value_display(&self) -> String {
        Money::from_cents(self.property_value_cents).display()
    }

    // Formats the value for display in templates.
    pub fn down_payment_display(&self) -> String {
        Money::from_cents(self.down_payment_cents).display()
    }

    // Formats the value for display in templates.
    pub fn loan_amount_display(&self) -> String {
        Money::from_cents(self.loan_amount_cents).display()
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
            "approved" => "Approved".to_string(),
            "rejected" => "Rejected".to_string(),
            "fully_paid" => "Fully Paid".to_string(),
            value => value.replace('_', " "),
        }
    }

    // Formats the value for display in templates.
    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y").to_string()
    }

    // Returns whether this record is payable.
    pub fn is_payable(&self) -> bool {
        self.status == "approved" && self.outstanding_cents > 0
    }

    // Returns whether this record is pending.
    pub fn is_pending(&self) -> bool {
        self.status == "pending"
    }

    // Explains the down-payment hold in customer-facing language.
    pub fn down_payment_note(&self) -> &'static str {
        match self.status.as_str() {
            "pending" => "Down payment is reserved while the bank reviews this application.",
            "rejected" => "Down payment was released back to your account.",
            "approved" | "fully_paid" => "Down payment remains applied to this approved home loan.",
            _ => "Down payment status follows this application record.",
        }
    }
}

#[derive(Debug, Clone)]
// Domain record used by services, repositories and templates.
pub struct HomeLoanSummary {
    pub total_outstanding_cents: i64,
    pub pending_count: usize,
    pub approved_count: usize,
}

impl HomeLoanSummary {
    // Builds the model value from applications.
    pub fn from_applications(applications: &[HomeLoanApplication]) -> Self {
        let total_outstanding_cents = applications
            .iter()
            .filter(|application| application.status == "approved")
            .map(|application| application.outstanding_cents)
            .sum();
        let pending_count = applications
            .iter()
            .filter(|application| application.status == "pending")
            .count();
        let approved_count = applications
            .iter()
            .filter(|application| application.status == "approved")
            .count();

        Self {
            total_outstanding_cents,
            pending_count,
            approved_count,
        }
    }

    // Formats the value for display in templates.
    pub fn outstanding_display(&self) -> String {
        Money::from_cents(self.total_outstanding_cents).display()
    }

    // Formats the value for display in templates.
    pub fn total_outstanding_display(&self) -> String {
        self.outstanding_display()
    }
}
