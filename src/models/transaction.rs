use super::formatting::title_case_code;
use super::Money;
use chrono::NaiveDateTime;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub product_id: Uuid,
    pub customer_id: Uuid,
    pub transaction_type: String,
    pub amount_cents: i64,
    pub balance_after_cents: i64,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
}

impl Transaction {
    pub fn transaction_type_display(&self) -> String {
        match self.transaction_type.as_str() {
            "DEPOSIT" => "Deposit".to_string(),
            "WITHDRAWAL" => "Withdrawal".to_string(),
            "TRANSFER_IN" => "Transfer In".to_string(),
            "TRANSFER_OUT" => "Transfer Out".to_string(),
            value => title_case_code(value),
        }
    }

    pub fn amount_display(&self) -> String {
        Money::from_cents(self.amount_cents).display()
    }

    pub fn balance_after_display(&self) -> String {
        Money::from_cents(self.balance_after_cents).display()
    }

    pub fn description_display(&self) -> &str {
        self.description.as_deref().unwrap_or("No description")
    }

    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y, %I:%M %p").to_string()
    }
}
