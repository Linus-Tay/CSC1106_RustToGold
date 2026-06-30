use super::Money;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
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
    pub fn principal_display(&self) -> String {
        Money::from_cents(self.principal_cents).display()
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
            "active" => "Active".to_string(),
            "rejected" => "Rejected".to_string(),
            "fully_paid" => "Fully Paid".to_string(),
            "cancelled" => "Cancelled".to_string(),
            value => value.replace('_', " "),
        }
    }

    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y").to_string()
    }

    pub fn is_payable(&self) -> bool {
        self.status == "active" && self.outstanding_cents > 0
    }
}
