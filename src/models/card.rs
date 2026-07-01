// Model layer: domain structs plus small display helpers used by services and templates.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
// Domain record used by services, repositories and templates.
pub struct Card {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub linked_product_id: Uuid,
    pub account_number: String,
    pub card_type: String,
    pub display_name: String,
    pub masked_number: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Card {
    // Formats the value for display in templates.
    pub fn card_type_display(&self) -> String {
        match self.card_type.as_str() {
            "debit" => "Everyday Debit Card".to_string(),
            "student" => "Campus Student Card".to_string(),
            value => value.replace('_', " "),
        }
    }

    // Formats the value for display in templates.
    pub fn status_display(&self) -> String {
        match self.status.as_str() {
            "active" => "Active".to_string(),
            "frozen" => "Frozen".to_string(),
            "cancelled" => "Cancelled".to_string(),
            value => value.to_string(),
        }
    }

    // Returns whether this record is active.
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }

    // Returns whether this record is frozen.
    pub fn is_frozen(&self) -> bool {
        self.status == "frozen"
    }

    // Formats the value for display in templates.
    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y").to_string()
    }
}
