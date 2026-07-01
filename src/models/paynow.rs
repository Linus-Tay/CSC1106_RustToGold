// Model layer: domain structs plus small display helpers used by services and templates.

use super::formatting::title_case_code;
use super::Money;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
// Domain record used by services, repositories and templates.
pub struct PayNowRegistration {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub paynow_type: String,
    pub paynow_id: String,
    pub linked_account_id: Uuid,
    pub account_number: String,
    pub product_id: String,
    pub balance_cents: i64,
    pub status: String,
    pub registered_at: DateTime<Utc>,
}

impl PayNowRegistration {
    // Formats the value for display in templates.
    pub fn paynow_type_display(&self) -> String {
        match self.paynow_type.as_str() {
            "phone_number" => "Phone Number".to_string(),
            "nric" => "NRIC/FIN".to_string(),
            value => title_case_code(value),
        }
    }

    // Formats the value for display in templates.
    pub fn identifier_display(&self) -> String {
        if self.paynow_type == "phone_number" && self.paynow_id.len() == 8 {
            format!("+65 {} {}", &self.paynow_id[0..4], &self.paynow_id[4..8])
        } else {
            self.paynow_id.clone()
        }
    }

    // Formats the value for display in templates.
    pub fn linked_product_display(&self) -> String {
        title_case_code(&self.product_id)
    }

    // Formats the value for display in templates.
    pub fn balance_display(&self) -> String {
        Money::from_cents(self.balance_cents).display()
    }

    // Formats the value for display in templates.
    pub fn status_display(&self) -> String {
        title_case_code(&self.status)
    }

    // Formats the value for display in templates.
    pub fn registered_at_display(&self) -> String {
        self.registered_at.format("%d %b %Y, %I:%M %p").to_string()
    }

    // Returns whether this record is active.
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }
}
