// Form layer: request payload structs received from HTML forms.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
// Form payload for the MonitoringStatusForm request.
pub struct MonitoringStatusForm {
    pub status: String,
    pub review_notes: Option<String>,
}
