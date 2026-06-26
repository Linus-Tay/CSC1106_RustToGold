use super::formatting::title_case_code;
use super::Money;
use chrono::{NaiveDateTime, DateTime, Utc};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub account_number: String,
    pub product_id: String,
    pub product_type: String,
    pub balance_cents: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Product {
    pub fn balance_display(&self) -> String {
        Money::from_cents(self.balance_cents).display()
    }

    pub fn product_id_display(&self) -> String {
        title_case_code(&self.product_id)
    }

    pub fn status_display(&self) -> String {
        title_case_code(self.status.as_str())
    }

    pub fn can_receive_deposit(&self, amount: Money) -> bool {
        self.status == "active" && amount.cents() > 0
    }

    pub fn get_customer_id(&self) -> Uuid {
        self.customer_id
    }
}

pub trait ProductWorkflow {
    fn is_open_for_customer_actions(&self) -> bool;
    fn projected_balance_after_deposit(&self, amount: Money) -> Option<Money>;
}

impl ProductWorkflow for Product {
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
