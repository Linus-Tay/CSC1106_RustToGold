use serde::Deserialize;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct AuditLogFilterForm {
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub entity_type: String,
    #[serde(default)]
    pub user_id: String,
}
