// Form layer: request payload structs received from HTML forms.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
// Form payload for the StatementRequest request.
pub struct StatementRequest {
    pub account_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}
