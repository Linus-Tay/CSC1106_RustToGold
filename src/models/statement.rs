use super::{Money, Product};
use chrono::{NaiveDate, NaiveDateTime};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct StatementTransaction {
    pub id: Uuid,
    pub transaction_type: String,
    pub amount_cents: i64,
    pub balance_after_cents: i64,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
}

impl StatementTransaction {
    pub fn date_display(&self) -> String {
        self.created_at.format("%d %b %Y").to_string()
    }

    pub fn time_display(&self) -> String {
        self.created_at.format("%I:%M %p").to_string()
    }

    pub fn transaction_type_display(&self) -> String {
        match self.transaction_type.as_str() {
            "deposit" => "Deposit".to_string(),
            "withdrawal" => "Withdrawal".to_string(),
            "transfer_in" => "Transfer In".to_string(),
            "transfer_out" => "Transfer Out".to_string(),
            "internal_transfer_in" => "Own Account Transfer In".to_string(),
            "internal_transfer_out" => "Own Account Transfer Out".to_string(),
            "paynow_transfer_in" => "PayNow In".to_string(),
            "paynow_transfer_out" => "PayNow Out".to_string(),
            "loan_disbursement" => "Loan Disbursement".to_string(),
            "loan_payment" => "Loan Payment".to_string(),
            "home_loan_payment" => "Home Loan Payment".to_string(),
            "fixed_deposit_open" => "Fixed Deposit Opened".to_string(),
            "fixed_deposit_withdrawal" => "Fixed Deposit Withdrawal".to_string(),
            "fixed_deposit_payout" => "Fixed Deposit Payout".to_string(),
            value => value.replace('_', " "),
        }
    }

    pub fn description_display(&self) -> String {
        self.description
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("RustToGold transaction")
            .to_string()
    }

    pub fn signed_amount_cents(&self) -> i64 {
        if self.is_credit() {
            self.amount_cents
        } else {
            -self.amount_cents
        }
    }

    pub fn is_credit(&self) -> bool {
        matches!(
            self.transaction_type.as_str(),
            "deposit" | "transfer_in" | "paynow_transfer_in" | "loan_disbursement" | "fixed_deposit_payout" | "fixed_deposit_withdrawal"
        )
    }

    pub fn debit_display(&self) -> String {
        if self.is_credit() {
            "-".to_string()
        } else {
            Money::from_cents(self.amount_cents).display()
        }
    }

    pub fn credit_display(&self) -> String {
        if self.is_credit() {
            Money::from_cents(self.amount_cents).display()
        } else {
            "-".to_string()
        }
    }

    pub fn balance_after_display(&self) -> String {
        Money::from_cents(self.balance_after_cents).display()
    }
}

#[derive(Debug, Clone)]
pub struct BankStatement {
    pub customer_name: String,
    pub customer_email: String,
    pub account: Product,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub generated_at: NaiveDateTime,
    pub opening_balance_cents: i64,
    pub closing_balance_cents: i64,
    pub transactions: Vec<StatementTransaction>,
}

impl BankStatement {
    pub fn account_number(&self) -> &str {
        &self.account.account_number
    }

    pub fn product_name(&self) -> String {
        self.account.product_id_display()
    }

    pub fn period_display(&self) -> String {
        format!(
            "{} to {}",
            self.start_date.format("%d %b %Y"),
            self.end_date.format("%d %b %Y")
        )
    }

    pub fn generated_at_display(&self) -> String {
        self.generated_at.format("%d %b %Y, %I:%M %p").to_string()
    }

    pub fn opening_balance_display(&self) -> String {
        Money::from_cents(self.opening_balance_cents).display()
    }

    pub fn closing_balance_display(&self) -> String {
        Money::from_cents(self.closing_balance_cents).display()
    }

    pub fn has_transactions(&self) -> bool {
        !self.transactions.is_empty()
    }
}
