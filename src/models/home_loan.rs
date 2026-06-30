use super::Money;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
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
    pub reviewed_by: Option<i64>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl HomeLoanApplication {
    pub fn property_value_display(&self) -> String {
        Money::from_cents(self.property_value_cents).display()
    }

    pub fn down_payment_display(&self) -> String {
        Money::from_cents(self.down_payment_cents).display()
    }

    pub fn loan_amount_display(&self) -> String {
        Money::from_cents(self.loan_amount_cents).display()
    }

    pub fn monthly_payment_display(&self) -> String {
        Money::from_cents(self.monthly_payment_cents).display()
    }

    pub fn outstanding_display(&self) -> String {
        Money::from_cents(self.outstanding_cents).display()
    }

    pub fn rate_display(&self) -> String {
        format!("{:.2}%", self.annual_rate_bps as f64 / 100.0)
    }

    pub fn status_display(&self) -> String {
        match self.status.as_str() {
            "pending" => "Pending Review".to_string(),
            "approved" => "Approved".to_string(),
            "rejected" => "Rejected".to_string(),
            "fully_paid" => "Fully Paid".to_string(),
            value => value.replace('_', " "),
        }
    }

    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y").to_string()
    }

    pub fn is_payable(&self) -> bool {
        self.status == "approved" && self.outstanding_cents > 0
    }

    pub fn is_pending(&self) -> bool {
        self.status == "pending"
    }
}

#[derive(Debug, Clone)]
pub struct HomeLoanSummary {
    pub total_outstanding_cents: i64,
    pub pending_count: usize,
    pub approved_count: usize,
}

impl HomeLoanSummary {
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

    pub fn outstanding_display(&self) -> String {
        Money::from_cents(self.total_outstanding_cents).display()
    }

    pub fn total_outstanding_display(&self) -> String {
        self.outstanding_display()
    }
}
