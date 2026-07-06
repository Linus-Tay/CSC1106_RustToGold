use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Card {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub linked_product_id: Uuid,
    pub account_number: String,
    pub card_type: String,
    pub pin_hash: String,
    pub display_name: String,
    pub card_number: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Card {
    // Format card type display
    pub fn card_type_display(&self) -> String {
        match self.card_type.as_str() {
            "debit" => "Everyday Debit Card".to_string(),
            "student" => "Campus Student Card".to_string(),
            value => value.replace('_', " "),
        }
    }

    // Format status display
    pub fn status_display(&self) -> String {
        match self.status.as_str() {
            "active" => "Active".to_string(),
            "frozen" => "Frozen".to_string(),
            "cancelled" => "Cancelled".to_string(),
            value => value.to_string(),
        }
    }

    // Check is active
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }

    // Check is frozen
    pub fn is_frozen(&self) -> bool {
        self.status == "frozen"
    }

    // Format created at display
    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y").to_string()
    }
}
