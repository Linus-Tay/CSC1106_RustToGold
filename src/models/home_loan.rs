use super::formatting::title_case_code;
use super::Money;
use chrono::{NaiveDate, NaiveDateTime};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct HomeLoanApplication {
    pub id: i64,
    pub user_id: i64,
    pub account_id: i64,
    pub house_type: String,
    pub requested_amount_cents: i64,
    pub interest_rate_bps: i32,
    pub term_months: i32,
    pub status: String,
    pub staff_remarks: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub approved_amount_cents: Option<i64>,
    pub approved_by: Option<i64>,
    pub approved_at: Option<NaiveDateTime>,
    pub total_repayment_cents: Option<i64>,
    pub remaining_cents: Option<i64>,
    pub monthly_payment_cents: Option<i64>,
    pub next_due_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AdminHomeLoanRecord {
    pub id: i64,
    pub user_id: i64,
    pub account_id: i64,
    pub house_type: String,
    pub requested_amount_cents: i64,
    pub interest_rate_bps: i32,
    pub term_months: i32,
    pub status: String,
    pub staff_remarks: Option<String>,
    pub approved_amount_cents: Option<i64>,
    pub approved_by: Option<i64>,
    pub approved_at: Option<NaiveDateTime>,
    pub total_repayment_cents: Option<i64>,
    pub remaining_cents: Option<i64>,
    pub monthly_payment_cents: Option<i64>,
    pub next_due_date: Option<NaiveDate>,
    pub customer_name: String,
    pub customer_email: String,
    pub account_number: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct HomeLoanSummary {
    pub total_count: i64,
    pub pending_count: i64,
    pub approved_count: i64,
    pub completed_count: i64,
    pub rejected_count: i64,
    pub total_approved_cents: i64,
    pub total_remaining_cents: i64,
    pub total_monthly_payment_cents: i64,
}

impl HomeLoanSummary {
    pub fn total_approved_display(&self) -> String {
        Money::from_cents(self.total_approved_cents).display()
    }

    pub fn total_remaining_display(&self) -> String {
        Money::from_cents(self.total_remaining_cents).display()
    }

    pub fn total_monthly_payment_display(&self) -> String {
        Money::from_cents(self.total_monthly_payment_cents).display()
    }
}

impl HomeLoanApplication {
    pub fn house_type_display(&self) -> String {
        match self.house_type.as_str() {
            "hdb_1_or_2_room" => "HDB 1- or 2-Room Flat".to_string(),
            "hdb_3_or_larger" => "HDB 3-Room or Larger Flat".to_string(),
            "condo" => "Private Condominium".to_string(),
            "landed" => "Landed Property".to_string(),
            value => title_case_code(value),
        }
    }

    pub fn requested_amount_display(&self) -> String {
        Money::from_cents(self.requested_amount_cents).display()
    }

    pub fn approved_amount_display(&self) -> String {
        self.approved_amount_cents
            .map(|value| Money::from_cents(value).display())
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn total_repayment_display(&self) -> String {
        self.total_repayment_cents
            .map(|value| Money::from_cents(value).display())
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn remaining_display(&self) -> String {
        self.remaining_cents
            .map(|value| Money::from_cents(value).display())
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn monthly_payment_display(&self) -> String {
        self.monthly_payment_cents
            .map(|value| Money::from_cents(value).display())
            .unwrap_or_else(|| "Pending approval".to_string())
    }


    pub fn next_due_date_display(&self) -> String {
        self.next_due_date
            .map(|value| value.format("%d %b %Y").to_string())
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn interest_rate_display(&self) -> String {
        format!("{:.2}%", self.interest_rate_bps as f64 / 100.0)
    }

    pub fn term_years_display(&self) -> String {
        format!("{} years", self.term_months / 12)
    }

    pub fn is_overdue(&self) -> bool {
        let today = chrono::Local::now().date_naive();

        self.status == "approved"
            && self.next_due_date
                .map(|date| date < today)
                .unwrap_or(false)
    }


    pub fn status_display(&self) -> String {
        if self.is_overdue() {
            return "Overdue".to_string();
        }

        match self.status.as_str() {
            "pending_review" => "Pending Review".to_string(),
            value => title_case_code(value),
        }
    }

    pub fn can_pay(&self) -> bool {
        self.status == "approved"
            && self.remaining_cents.unwrap_or(0) > 0
            && self.monthly_payment_cents.unwrap_or(0) > 0
    }
}


impl AdminHomeLoanRecord {
    pub fn house_type_display(&self) -> String {
        match self.house_type.as_str() {
            "hdb_1_or_2_room" => "HDB 1- or 2-Room Flat".to_string(),
            "hdb_3_or_larger" => "HDB 3-Room or Larger Flat".to_string(),
            "condo" => "Private Condominium".to_string(),
            "landed" => "Landed Property".to_string(),
            value => title_case_code(value),
        }
    }

    pub fn requested_amount_display(&self) -> String {
        Money::from_cents(self.requested_amount_cents).display()
    }

    pub fn approved_amount_display(&self) -> String {
        self.approved_amount_cents
            .map(|value| Money::from_cents(value).display())
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn remaining_display(&self) -> String {
        self.remaining_cents
            .map(|value| Money::from_cents(value).display())
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn monthly_payment_display(&self) -> String {
        self.monthly_payment_cents
            .map(|value| Money::from_cents(value).display())
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn is_overdue(&self) -> bool {
        let today = chrono::Local::now().date_naive();

        self.status == "approved"
            && self.next_due_date
                .map(|date| date < today)
                .unwrap_or(false)
    }

    pub fn status_display(&self) -> String {
        if self.is_overdue() {
            return "Overdue".to_string();
        }

        match self.status.as_str() {
            "pending_review" => "Pending Review".to_string(),
            value => title_case_code(value),
        }
    }
}