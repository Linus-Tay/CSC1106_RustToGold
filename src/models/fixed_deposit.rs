use super::formatting::title_case_code;
use super::Money;
use chrono::{NaiveDate, NaiveDateTime};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct FixedDepositPlan {
    pub id: i64,
    pub name: String,
    pub duration_months: i32,
    pub interest_rate_bps: i32,
    pub minimum_amount_cents: i64,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl FixedDepositPlan {
    pub fn interest_rate_display(&self) -> String {
        format!("{:.2}%", self.interest_rate_bps as f64 / 100.0)
    }

    pub fn interest_rate_input_value(&self) -> String {
        format!("{:.2}", self.interest_rate_bps as f64 / 100.0)
    }

    pub fn minimum_amount_display(&self) -> String {
        Money::from_cents(self.minimum_amount_cents).display()
    }

    pub fn minimum_amount_input_value(&self) -> String {
        Money::from_cents(self.minimum_amount_cents)
            .display()
            .trim_start_matches('$')
            .to_string()
    }

    pub fn status_display(&self) -> String {
        title_case_code(&self.status)
    }

    pub fn is_active(&self) -> bool {
        self.status == "active"
    }

    pub fn is_inactive(&self) -> bool {
        !self.is_active()
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct FixedDeposit {
    pub id: i64,
    pub user_id: i64,
    pub account_id: i64,
    pub plan_id: i64,
    pub principal_cents: i64,
    pub interest_rate_bps: i32,
    pub interest_cents: i64,
    pub penalty_cents: i64,
    pub payout_cents: i64,
    pub start_date: NaiveDate,
    pub maturity_date: NaiveDate,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl FixedDeposit {
    pub fn principal_display(&self) -> String {
        Money::from_cents(self.principal_cents).display()
    }

    pub fn interest_rate_display(&self) -> String {
        format!("{:.2}%", self.interest_rate_bps as f64 / 100.0)
    }

    pub fn interest_display(&self) -> String {
        Money::from_cents(self.interest_cents).display()
    }

    pub fn penalty_display(&self) -> String {
        Money::from_cents(self.penalty_cents).display()
    }

    pub fn payout_display(&self) -> String {
        Money::from_cents(self.payout_cents).display()
    }

    pub fn start_date_display(&self) -> String {
        self.start_date.format("%d %b %Y").to_string()
    }

    pub fn maturity_date_display(&self) -> String {
        self.maturity_date.format("%d %b %Y").to_string()
    }

    pub fn status_display(&self) -> String {
        title_case_code(&self.status)
    }

    pub fn status_badge_class(&self) -> &'static str {
        match self.status.as_str() {
            "active" => "badge--success",
            "matured" => "badge--info",
            "withdrawn" => "badge--muted",
            "paid_out" => "badge--success",
            "cancelled" => "badge--danger",
            _ => "badge--muted",
        }
    }

    pub fn can_be_withdrawn(&self) -> bool {
        self.status == "active" || self.status == "matured"
    }

    pub fn action_label(&self) -> &'static str {
        if self.status == "matured" {
            "Claim Payout"
        } else {
            "Withdraw Early"
        }
    }

    pub fn payout_label(&self) -> &'static str {
        if self.status == "withdrawn" || self.status == "paid_out" {
            "Final payout"
        } else {
            "Expected payout"
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct FixedDepositSummary {
    pub total_count: i64,
    pub active_count: i64,
    pub matured_count: i64,
    pub total_principal_cents: i64,
    pub total_interest_cents: i64,
    pub total_payout_cents: i64,
}

impl FixedDepositSummary {
    pub fn total_principal_display(&self) -> String {
        Money::from_cents(self.total_principal_cents).display()
    }

    pub fn total_interest_display(&self) -> String {
        Money::from_cents(self.total_interest_cents).display()
    }

    pub fn total_payout_display(&self) -> String {
        Money::from_cents(self.total_payout_cents).display()
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct AdminFixedDepositRecord {
    pub id: i64,
    pub user_id: i64,
    pub account_id: i64,
    pub plan_id: i64,
    pub principal_cents: i64,
    pub interest_rate_bps: i32,
    pub interest_cents: i64,
    pub penalty_cents: i64,
    pub payout_cents: i64,
    pub start_date: NaiveDate,
    pub maturity_date: NaiveDate,
    pub status: String,
    pub customer_name: String,
    pub customer_email: String,
    pub account_number: String,
    pub plan_name: String,
}

impl AdminFixedDepositRecord {
    pub fn principal_display(&self) -> String {
        Money::from_cents(self.principal_cents).display()
    }

    pub fn interest_rate_display(&self) -> String {
        format!("{:.2}%", self.interest_rate_bps as f64 / 100.0)
    }

    pub fn interest_display(&self) -> String {
        Money::from_cents(self.interest_cents).display()
    }

    pub fn penalty_display(&self) -> String {
        Money::from_cents(self.penalty_cents).display()
    }

    pub fn payout_display(&self) -> String {
        Money::from_cents(self.payout_cents).display()
    }

    pub fn maturity_date_display(&self) -> String {
        self.maturity_date.format("%d %b %Y").to_string()
    }

    pub fn status_display(&self) -> String {
        title_case_code(&self.status)
    }

    pub fn status_badge_class(&self) -> &'static str {
        match self.status.as_str() {
            "active" => "badge--success",
            "matured" => "badge--info",
            "withdrawn" => "badge--muted",
            "paid_out" => "badge--success",
            "cancelled" => "badge--danger",
            _ => "badge--muted",
        }
    }
}
