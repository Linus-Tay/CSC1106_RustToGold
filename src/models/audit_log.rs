use super::formatting::title_case_code;
use super::Money;
use chrono::NaiveDateTime;
use serde_json::Value as JsonValue;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct AuditLogEntry {
    pub id: i64,
    pub user_id: Option<i64>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub old_value: Option<JsonValue>,
    pub new_value: Option<JsonValue>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: String,
    pub created_at: NaiveDateTime,
    // Joined from users table (nullable since user_id can be NULL / actor can be deleted)
    pub actor_name: Option<String>,
    pub actor_email: Option<String>,
}

impl AuditLogEntry {
    pub fn action_display(&self) -> String {
        match self.action.as_str() {
            "create_staff" => "Created Staff".to_string(),
            "update_staff" => "Updated Staff".to_string(),
            "delete_staff" => "Deleted Staff".to_string(),
            "freeze_account" => "Froze Account".to_string(),
            "create_fixed_deposit" => "Created Fixed Deposit".to_string(),
            "update_fixed_deposit" => "Updated Fixed Deposit".to_string(),
            "delete_fixed_deposit" => "Deleted Fixed Deposit".to_string(),
            other => other.to_string(),
        }
    }

    pub fn status_display(&self) -> String {
        match self.status.as_str() {
            "success" => "Success".to_string(),
            "failed" => "Failed".to_string(),
            "unauthorized" => "Unauthorized".to_string(),
            other => other.to_string(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.status == "success"
    }

    pub fn is_failed(&self) -> bool {
        self.status == "failed"
    }

    pub fn is_unauthorized(&self) -> bool {
        self.status == "unauthorized"
    }

    pub fn actor_display(&self) -> String {
        match (&self.actor_name, self.user_id) {
            (Some(name), _) => name.clone(),
            (None, Some(id)) => format!("Deleted User (#{})", id),
            (None, None) => "System".to_string(),
        }
    }

    pub fn entity_display(&self) -> String {
        match self.entity_id {
            Some(id) => format!("{} #{}", self.entity_type, id),
            None => self.entity_type.clone(),
        }
    }

    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y %H:%M:%S").to_string()
    }

    pub fn old_value_display(&self) -> String {
        match &self.old_value {
            Some(v) => serde_json::to_string_pretty(v).unwrap_or_default(),
            None => String::new(),
        }
    }

    pub fn new_value_display(&self) -> String {
        match &self.new_value {
            Some(v) => serde_json::to_string_pretty(v).unwrap_or_default(),
            None => String::new(),
        }
    }

    pub fn has_old_value(&self) -> bool {
        self.old_value.is_some()
    }

    pub fn has_new_value(&self) -> bool {
        self.new_value.is_some()
    }

    pub fn ip_address_display(&self) -> String {
        self.ip_address.clone().unwrap_or_else(|| "—".to_string())
    }
}
