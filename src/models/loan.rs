use super::formatting::title_case_code;
use super::Money;
use chrono::{NaiveDate, NaiveDateTime};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Loan {
    pub id: i64,
    pub user_id: i64,
    pub account_id: i64,
    pub principal_cents: i64,
    pub interest_rate_bps: i32,
    pub interest_cents: i64,
    pub total_repayment_cents: i64,
    pub remaining_cents: i64,
    pub monthly_payment_cents: i64,
    pub term_months: i32,
    pub next_due_date: NaiveDate,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Loan {
    pub fn principal_display(&self) -> String {
        Money::from_cents(self.principal_cents).display()
    }

    pub fn interest_rate_display(&self) -> String {
        format!("{:.2}%", self.interest_rate_bps as f64 / 100.0)
    }

    pub fn interest_display(&self) -> String {
        Money::from_cents(self.interest_cents).display()
    }

    pub fn remaining_display(&self) -> String {
        Money::from_cents(self.remaining_cents).display()
    }

    pub fn monthly_payment_display(&self) -> String {
        Money::from_cents(self.monthly_payment_cents).display()
    }

    pub fn next_due_date_display(&self) -> String {
        self.next_due_date.format("%d %b %Y").to_string()
    }

    pub fn status_display(&self) -> String {
        title_case_code(&self.status)
    }

    pub fn can_pay(&self) -> bool {
        self.status == "active" && self.remaining_cents > 0
    }

    pub fn display_status(&self) -> String {
    let today = chrono::Utc::now().date_naive();

    if self.status == "active" && self.next_due_date < today {
        "Overdue".to_string()
    } else {
        title_case_code(&self.status)
    }}

    pub fn calculate_max_borrowing_limit(monthly_income_cents: i64) -> i64 {
    monthly_income_cents * 4
    }

    pub fn is_overdue(&self) -> bool {
    let today = chrono::Utc::now().date_naive();
    self.status == "active" && self.next_due_date < today
    }



}

pub struct SimpleLoanCalculator;

impl SimpleLoanCalculator {
    pub fn calculate_interest_cents(
        principal_cents: i64,
        annual_rate_bps: i32,
        term_months: i32,
    ) -> i64 {
        let numerator = principal_cents as i128 * annual_rate_bps as i128 * term_months as i128;
        let denominator = 10_000_i128 * 12_i128;
        (numerator / denominator) as i64
    }

    pub fn calculate_monthly_payment_cents(total_repayment_cents: i64, term_months: i32) -> i64 {
        (total_repayment_cents + term_months as i64 - 1) / term_months as i64
    }
}

