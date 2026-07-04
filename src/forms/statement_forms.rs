use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct StatementRequest {
    pub account_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}
