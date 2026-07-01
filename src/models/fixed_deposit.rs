// Model layer: domain structs plus small display helpers used by services and templates.

use super::Money;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
// Domain record used by services, repositories and templates.
pub struct FixedDepositPlan {
    pub id: i64,
    pub plan_name: String,
    pub tenure_months: i32,
    pub annual_rate_bps: i32,
    pub minimum_amount_cents: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FixedDepositPlan {
    // Formats the value for display in templates.
    pub fn rate_display(&self) -> String {
        format!("{:.2}%", self.annual_rate_bps as f64 / 100.0)
    }

    // Formats the value for display in templates.
    pub fn minimum_amount_display(&self) -> String {
        Money::from_cents(self.minimum_amount_cents).display()
    }

    // Returns the raw value used by form fields and validation.
    pub fn minimum_amount_plain(&self) -> String {
        format!("{}.{:02}", self.minimum_amount_cents / 100, self.minimum_amount_cents.abs() % 100)
    }

    // Formats the value for display in templates.
    pub fn status_display(&self) -> &'static str {
        if self.is_active { "Active" } else { "Inactive" }
    }
}

#[derive(Debug, Clone, FromRow)]
// Domain record used by services, repositories and templates.
pub struct FixedDeposit {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub funding_product_id: Uuid,
    pub plan_id: i64,
    pub plan_name: String,
    pub principal_cents: i64,
    pub annual_rate_bps: i32,
    pub tenure_months: i32,
    pub interest_cents: i64,
    pub maturity_date: NaiveDate,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FixedDeposit {
    // Formats the value for display in templates.
    pub fn principal_display(&self) -> String {
        Money::from_cents(self.principal_cents).display()
    }

    // Formats the value for display in templates.
    pub fn interest_display(&self) -> String {
        Money::from_cents(self.interest_cents).display()
    }

    // Formats the value for display in templates.
    pub fn payout_display(&self) -> String {
        Money::from_cents(self.principal_cents + self.interest_cents).display()
    }

    // Formats the value for display in templates.
    pub fn rate_display(&self) -> String {
        format!("{:.2}%", self.annual_rate_bps as f64 / 100.0)
    }

    // Formats the value for display in templates.
    pub fn maturity_date_display(&self) -> String {
        self.maturity_date.format("%d %b %Y").to_string()
    }

    // Formats the value for display in templates.
    pub fn status_display(&self) -> String {
        match self.status.as_str() {
            "active" => "Active".to_string(),
            "matured" => "Matured".to_string(),
            "withdrawn" => "Withdrawn".to_string(),
            "paid_out" => "Paid Out".to_string(),
            value => value.replace('_', " "),
        }
    }

    // Returns whether this record can withdraw.
    pub fn can_withdraw(&self) -> bool {
        self.status == "active" || self.status == "matured"
    }
}

#[derive(Debug, Clone)]
// Domain record used by services, repositories and templates.
pub struct FixedDepositSummary {
    pub active_count: usize,
    pub matured_count: usize,
    pub total_principal_cents: i64,
    pub total_expected_interest_cents: i64,
}

impl FixedDepositSummary {
    // Builds the model value from fixed deposits.
    pub fn from_fixed_deposits(fixed_deposits: &[FixedDeposit]) -> Self {
        let active_count = fixed_deposits
            .iter()
            .filter(|fd| fd.status == "active")
            .count();
        let matured_count = fixed_deposits
            .iter()
            .filter(|fd| fd.status == "matured")
            .count();
        let total_principal_cents = fixed_deposits
            .iter()
            .filter(|fd| fd.status == "active" || fd.status == "matured")
            .map(|fd| fd.principal_cents)
            .sum();
        let total_expected_interest_cents = fixed_deposits
            .iter()
            .filter(|fd| fd.status == "active" || fd.status == "matured")
            .map(|fd| fd.interest_cents)
            .sum();

        Self {
            active_count,
            matured_count,
            total_principal_cents,
            total_expected_interest_cents,
        }
    }

    // Formats the value for display in templates.
    pub fn total_principal_display(&self) -> String {
        Money::from_cents(self.total_principal_cents).display()
    }

    // Formats the value for display in templates.
    pub fn active_principal_display(&self) -> String {
        self.total_principal_display()
    }

    // Formats the value for display in templates.
    pub fn expected_interest_display(&self) -> String {
        Money::from_cents(self.total_expected_interest_cents).display()
    }

    // Formats the value for display in templates.
    pub fn projected_interest_display(&self) -> String {
        self.expected_interest_display()
    }
}

#[derive(Debug, Clone, FromRow)]
// Domain record used by services, repositories and templates.
pub struct FixedDepositAdminRecord {
    pub id: Uuid,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: String,
    pub customer_nric: String,
    pub account_number: String,
    pub account_balance_cents: i64,
    pub plan_name: String,
    pub principal_cents: i64,
    pub annual_rate_bps: i32,
    pub tenure_months: i32,
    pub interest_cents: i64,
    pub maturity_date: NaiveDate,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl FixedDepositAdminRecord {
    // Formats the value for display in templates.
    pub fn principal_display(&self) -> String {
        Money::from_cents(self.principal_cents).display()
    }

    // Formats the value for display in templates.
    pub fn interest_display(&self) -> String {
        Money::from_cents(self.interest_cents).display()
    }

    // Formats the value for display in templates.
    pub fn payout_display(&self) -> String {
        Money::from_cents(self.principal_cents + self.interest_cents).display()
    }

    // Formats the value for display in templates.
    pub fn account_balance_display(&self) -> String {
        Money::from_cents(self.account_balance_cents).display()
    }

    // Formats the value for display in templates.
    pub fn rate_display(&self) -> String {
        format!("{:.2}%", self.annual_rate_bps as f64 / 100.0)
    }

    // Formats the value for display in templates.
    pub fn maturity_date_display(&self) -> String {
        self.maturity_date.format("%d %b %Y").to_string()
    }

    // Formats the value for display in templates.
    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y").to_string()
    }

    // Formats the value for display in templates.
    pub fn status_display(&self) -> String {
        match self.status.as_str() {
            "active" => "Active".to_string(),
            "matured" => "Matured".to_string(),
            "withdrawn" => "Withdrawn".to_string(),
            "paid_out" => "Paid Out".to_string(),
            value => value.replace('_', " "),
        }
    }
}
