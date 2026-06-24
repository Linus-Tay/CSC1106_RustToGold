use super::formatting::title_case_code;
use super::Money;
use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct BankAccount {
    pub id: i64,
    pub user_id: i64,
    pub account_number: String,
    pub account_type: String,
    pub balance_cents: i64,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl BankAccount {
    pub fn balance_display(&self) -> String {
        Money::from_cents(self.balance_cents).display()
    }

    pub fn account_type_display(&self) -> String {
        title_case_code(&self.account_type)
    }

    pub fn status_display(&self) -> String {
        title_case_code(&self.status)
    }

    pub fn can_receive_deposit(&self, amount: Money) -> bool {
        self.status == "active" && amount.cents() > 0
    }
}

pub trait AccountWorkflow {
    fn is_open_for_customer_actions(&self) -> bool;
    fn projected_balance_after_deposit(&self, amount: Money) -> Option<Money>;
}

impl AccountWorkflow for BankAccount {
    fn is_open_for_customer_actions(&self) -> bool {
        self.status == "active"
    }

    fn projected_balance_after_deposit(&self, amount: Money) -> Option<Money> {
        if !self.can_receive_deposit(amount) {
            return None;
        }

        self.balance_cents
            .checked_add(amount.cents())
            .map(Money::from_cents)
    }
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct BankAccountWithUser {
    // BankAccount fields
    pub id: i64,
    pub user_id: i64,
    pub account_number: String,
    pub account_type: String,
    pub balance_cents: i64,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    // Joined user fields
    pub full_name: String,
    pub email: String,
}

impl BankAccountWithUser {
    pub fn balance_display(&self) -> String {
        Money::from_cents(self.balance_cents).display()
    }

    pub fn account_type_display(&self) -> String {
        title_case_code(&self.account_type)
    }

    pub fn status_display(&self) -> String {
        title_case_code(&self.status)
    }
}