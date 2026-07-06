use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Customer {
    pub id: Uuid,
    pub full_name: String,
    pub nric: String,
    pub date_of_birth: NaiveDate,
    pub gender: String,
    pub nationality: String,
    pub residency: String,
    pub race: Option<String>,
    pub email: String,
    pub phone_number: String,
    pub residential_address: String,
    pub mailing_address: Option<String>,
    pub preferred_contact: Option<String>,
    pub employment_status: String,
    pub occupation: Option<String>,
    pub employer_name: Option<String>,
    pub industry: Option<String>,
    pub monthly_income_range: Option<String>,
    pub kyc_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Customer {
    // Format date of birth display
    pub fn date_of_birth_display(&self) -> String {
        self.date_of_birth.format("%d %b %Y").to_string()
    }

    // Format joined display
    pub fn joined_display(&self) -> String {
        self.created_at.format("%d %b %Y").to_string()
    }
}
