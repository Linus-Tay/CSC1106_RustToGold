// Model layer: domain structs plus small display helpers used by services and templates.

use super::{formatting::title_case_code, Money};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
// Domain record used by services, repositories and templates.
pub struct TransactionControl {
    pub customer_id: Uuid,
    pub daily_limit_cents: i64,
    pub pending_daily_limit_cents: Option<i64>,
    pub limit_change_effective_at: Option<DateTime<Utc>>,
    pub money_lock_enabled: bool,
    pub unlock_requested_at: Option<DateTime<Utc>>,
    pub unlock_effective_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TransactionControl {
    // Formats the value for display in templates.
    pub fn daily_limit_display(&self) -> String {
        Money::from_cents(self.daily_limit_cents).display()
    }

    // Formats the value for display in templates.
    pub fn pending_daily_limit_display(&self) -> String {
        self.pending_daily_limit_cents
            .map(Money::from_cents)
            .map(|money| money.display())
            .unwrap_or_else(|| "No pending change".to_string())
    }

    // Returns whether this record has pending limit change.
    pub fn has_pending_limit_change(&self) -> bool {
        self.pending_daily_limit_cents.is_some() && self.limit_change_effective_at.is_some()
    }

    // Returns whether this record has limit cooldown.
    pub fn has_limit_cooldown(&self) -> bool {
        self.limit_change_effective_at
            .map(|value| value > Utc::now())
            .unwrap_or(false)
    }

    // Formats the value for display in templates.
    pub fn limit_change_effective_display(&self) -> String {
        self.limit_change_effective_at
            .map(|value| value.format("%d %b %Y, %I:%M %p").to_string())
            .unwrap_or_else(|| "No cooldown active".to_string())
    }

    // Formats the value for display in templates.
    pub fn money_lock_status_display(&self) -> String {
        if self.money_lock_enabled {
            "Locked".to_string()
        } else {
            "Unlocked".to_string()
        }
    }

    // Returns whether this record has unlock cooldown.
    pub fn has_unlock_cooldown(&self) -> bool {
        self.money_lock_enabled && self.unlock_effective_at.is_some()
    }

    // Formats the value for display in templates.
    pub fn unlock_effective_display(&self) -> String {
        self.unlock_effective_at
            .map(|value| value.format("%d %b %Y, %I:%M %p").to_string())
            .unwrap_or_else(|| "No unlock request".to_string())
    }
}

#[derive(Debug, Clone, FromRow)]
// Domain record used by services, repositories and templates.
pub struct FraudAlert {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub product_id: Option<Uuid>,
    pub rule_code: String,
    pub severity: String,
    pub channel: String,
    pub amount_cents: i64,
    pub message: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl FraudAlert {
    // Formats the value for display in templates.
    pub fn amount_display(&self) -> String {
        Money::from_cents(self.amount_cents).display()
    }

    // Formats the value for display in templates.
    pub fn severity_display(&self) -> String {
        title_case_code(&self.severity)
    }

    // Formats the value for display in templates.
    pub fn status_display(&self) -> String {
        title_case_code(&self.status)
    }

    // Formats the value for display in templates.
    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y, %I:%M %p").to_string()
    }
}
