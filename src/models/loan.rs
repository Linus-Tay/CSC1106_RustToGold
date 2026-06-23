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
    pub fn interest_rate_display(&self) -> String {
        format!("{:.2}%", self.interest_rate_bps as f64 / 100.0)
    }

    pub fn remaining_loan_display(&self) -> String {
        Money::from_cents(self.remaining_cents).display()
    }

    pub fn monthly_payment_display(&self) -> String {
        Money::from_cents(self.monthly_payment_cents).display()
    }

    pub fn principal_loan_display(&self) -> String {
        Money::from_cents(self.principal_cents).display()
    }

    pub fn interest_amount_display(&self) -> String {
        Money::from_cents(self.interest_cents).display()
    }

    pub fn total_repayment_display(&self) -> String {
        Money::from_cents(self.total_repayment_cents).display()
    }

    pub fn loan_due_date_display(&self) -> String {
        self.next_due_date.format("%d %b %Y").to_string()
    }

    pub fn loan_status_display(&self) -> String {
        if self.loan_overdue() {
            "Overdue".to_string()
        } else {
            title_case_code(&self.status)
        }
    }

    pub fn loan_overdue(&self) -> bool {
        let today = chrono::Local::now().date_naive();
        self.status == "active" && self.next_due_date < today
    }

    pub fn can_pay(&self) -> bool {
        self.status == "active" && self.remaining_cents > 0
    }
}