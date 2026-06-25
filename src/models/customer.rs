use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::{Serialize, Deserialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Type)]
#[sqlx(type_name = "gender_type", rename_all = "UPPERCASE")]
pub enum Gender {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, Type, Serialize, Deserialize)]
#[sqlx(type_name = "residency_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Residency {
    Citizen,
    Pr,
    Foreigner,
}

#[derive(Debug, Clone, Copy, Type)]
#[sqlx(type_name = "employment_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmploymentType {
    Employed,
    SelfEmployed,
    Unemployed,
    Student,
    Retired,
}

#[derive(Debug, Clone, Copy, Type)]
#[sqlx(type_name = "contact_method_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContactMethod {
    Email,
    Phone,
}

#[derive(Debug, Clone, Copy, Type)]
#[sqlx(type_name = "kyc_status_type", rename_all = "UPPERCASE")]
pub enum KycStatus {
    PENDING,
    APPROVED,
    REJECTED,
}

#[derive(Debug, Clone, FromRow)]
pub struct Customer {
    pub id: Uuid,
    pub full_name: String,
    pub nric: String,
    pub date_of_birth: NaiveDate,
    pub gender: Gender,
    pub nationality: String,
    pub residency: Residency,
    pub race: Option<String>,
    pub email: String,
    pub phone_number: String,
    pub residential_address: String,
    pub mailing_address: Option<String>,
    pub preferred_contact: ContactMethod,
    pub employment_status: EmploymentType,
    pub occupation: Option<String>,
    pub employer_name: Option<String>,
    pub industry: Option<String>,
    pub monthly_income_range: Option<String>,
    pub kyc_status: KycStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Customer {
    pub fn date_of_birth_display(&self) -> String {
        self.date_of_birth.format("%d %b %Y").to_string()
    }

    pub fn joined_display(&self) -> String {
        self.created_at.format("%d %b %Y").to_string()
    }
}
