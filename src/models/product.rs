use super::formatting::title_case_code;
use super::Money;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
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
    // Format id display
    pub fn id_display(&self) -> String {
        self.id.to_string()
    }

    // Format balance display
    pub fn balance_display(&self) -> String {
        Money::from_cents(self.balance_cents).display()
    }

    // Format product id display
    pub fn product_id_display(&self) -> String {
        title_case_code(&self.product_id)
    }

    // Format status display
    pub fn status_display(&self) -> String {
        title_case_code(self.status.as_str())
    }

    // Check can receive deposit
    pub fn can_receive_deposit(&self, amount: Money) -> bool {
        self.status == "active" && amount.cents() > 0
    }

    // Return get customer id
    pub fn get_customer_id(&self) -> Uuid {
        self.customer_id
    }
}

pub trait ProductWorkflow {
    // Check is open for customer actions
    fn is_open_for_customer_actions(&self) -> bool;
    // Calculates the projected balance after deposit without mutating the record
    fn projected_balance_after_deposit(&self, amount: Money) -> Option<Money>;
}

impl ProductWorkflow for Product {
    // Check is open for customer actions
    fn is_open_for_customer_actions(&self) -> bool {
        self.status == "active"
    }

    // Calculates the projected balance after deposit without mutating the record
    fn projected_balance_after_deposit(&self, amount: Money) -> Option<Money> {
        if !self.can_receive_deposit(amount) {
            return None;
        }

        self.balance_cents
            .checked_add(amount.cents())
            .map(Money::from_cents)
    }
}
