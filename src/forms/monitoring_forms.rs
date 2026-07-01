use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MonitoringStatusForm {
    pub status: String,
    pub review_notes: Option<String>,
}
