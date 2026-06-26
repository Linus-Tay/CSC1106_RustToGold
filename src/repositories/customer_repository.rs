use crate::models::customer::{Customer};
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

pub struct NewCustomer<'a> {
    pub full_name: &'a str,
    pub nric: &'a str,
    pub date_of_birth: NaiveDate,
    pub gender: &'a str,
    pub nationality: &'a str,
    pub residency: &'a str,
    pub race: Option<&'a str>,
    pub email: &'a str,
    pub phone_number: &'a str,
    pub residential_address: &'a str,
    pub mailing_address: Option<&'a str>,
    pub preferred_contact: Option<&'a str>,
    pub employment_status: &'a str,
    pub occupation: Option<&'a str>,
    pub employer_name: Option<&'a str>,
    pub industry: Option<&'a str>,
    pub monthly_income_range: Option<&'a str>,
    pub kyc_status: Option<&'a str>,
}

pub async fn get_customer_by_nric(db: &PgPool, nric: String) -> Result<Option<Customer>, sqlx::Error> {
    println!("{}", nric);
    sqlx::query_as::<_, Customer>(
        r#"SELECT id, full_name, nric, date_of_birth, gender, nationality, residency, race,
                  email, phone_number, residential_address, mailing_address, preferred_contact,
                  employment_status, occupation, employer_name, industry, monthly_income_range,
                  kyc_status, created_at, updated_at FROM customers WHERE nric = $1"#
    )
    .bind(nric)
    .fetch_optional(db)
    .await
}

pub async fn create_customer(db: &PgPool, new_customer: &NewCustomer<'_>) -> Result<Customer, sqlx::Error> {
    sqlx::query_as::<_, Customer> (
        r#"
        INSERT INTO customers (
            full_name,
            nric,
            date_of_birth,
            gender,
            nationality,
            residency,
            race,
            email,
            phone_number,
            residential_address,
            mailing_address,
            preferred_contact,
            employment_status,
            occupation,
            employer_name,
            industry,
            monthly_income_range
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        RETURNING id, full_name, nric, date_of_birth, gender, nationality, residency, race,
                  email, phone_number, residential_address, mailing_address, preferred_contact,
                  employment_status, occupation, employer_name, industry, monthly_income_range,
                  kyc_status, created_at, updated_at
        "#,
    )
    .bind(new_customer.full_name)
    .bind(new_customer.nric)
    .bind(new_customer.date_of_birth)
    .bind(new_customer.gender)
    .bind(new_customer.nationality)
    .bind(new_customer.residency)
    .bind(new_customer.race)
    .bind(new_customer.email)
    .bind(new_customer.phone_number)
    .bind(new_customer.residential_address)
    .bind(new_customer.mailing_address)
    .bind(new_customer.preferred_contact)
    .bind(new_customer.employment_status)
    .bind(new_customer.occupation)
    .bind(new_customer.employer_name)
    .bind(new_customer.industry)
    .bind(new_customer.monthly_income_range)
    .fetch_one(db)
    .await
}

pub async fn update_customer(db: &PgPool, uuid: Uuid, new_info: &NewCustomer<'_>) -> Result<Customer, sqlx::Error> {
    println!("{}", uuid);
    sqlx::query_as::<_, Customer> (
        r#"
        UPDATE CUSTOMERS SET
            full_name = $1,
            nric = $2,
            date_of_birth = $3,
            gender = $4,
            nationality = $5,
            residency = $6,
            race = $7,
            email = $8,
            phone_number = $9,
            residential_address = $10,
            mailing_address = $11,
            preferred_contact = $12,
            employment_status = $13,
            occupation = $14,
            employer_name = $15,
            industry = $16,
            monthly_income_range = $17,
            kyc_status = $18,
            updated_at = NOW()
            WHERE id = $19
            RETURNING id, full_name, nric, date_of_birth, gender, nationality, residency, race,
            email, phone_number, residential_address, mailing_address, preferred_contact,
            employment_status, occupation, employer_name, industry, monthly_income_range,
            kyc_status, created_at, updated_at
        "#,
    )
    .bind(new_info.full_name)
    .bind(new_info.nric)
    .bind(new_info.date_of_birth)
    .bind(new_info.gender)
    .bind(new_info.nationality)
    .bind(new_info.residency)
    .bind(new_info.race)
    .bind(new_info.email)
    .bind(new_info.phone_number)
    .bind(new_info.residential_address)
    .bind(new_info.mailing_address)
    .bind(new_info.preferred_contact)
    .bind(new_info.employment_status)
    .bind(new_info.occupation)
    .bind(new_info.employer_name)
    .bind(new_info.industry)
    .bind(new_info.monthly_income_range)
    .bind(new_info.kyc_status.unwrap_or("pending"))
    .bind(uuid)
    .fetch_one(db)
    .await
}
pub async fn create_customer_profile_for_user(
    db: &PgPool,
    customer_id: Uuid,
    full_name: &str,
    nric: &str,
    date_of_birth: NaiveDate,
    nationality: &str,
    residency: &str,
    email: &str,
    phone_number: &str,
    residential_address: &str,
    mailing_address: Option<&str>,
    employment_status: &str,
    occupation: Option<&str>,
    employer_name: Option<&str>,
    monthly_income_range: Option<&str>,
) -> Result<Customer, sqlx::Error> {
    sqlx::query_as::<_, Customer>(
        r#"
        INSERT INTO customers (
            id,
            full_name,
            nric,
            date_of_birth,
            gender,
            nationality,
            residency,
            race,
            email,
            phone_number,
            residential_address,
            mailing_address,
            preferred_contact,
            employment_status,
            occupation,
            employer_name,
            industry,
            monthly_income_range,
            kyc_status
        )
        VALUES ($1, $2, $3, $4, 'Not collected', $5, $6, NULL, $7, $8, $9, $10, NULL, $11, $12, $13, NULL, $14, 'pending')
        RETURNING id, full_name, nric, date_of_birth, gender, nationality, residency, race,
                  email, phone_number, residential_address, mailing_address, preferred_contact,
                  employment_status, occupation, employer_name, industry, monthly_income_range,
                  kyc_status, created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .bind(full_name)
    .bind(nric)
    .bind(date_of_birth)
    .bind(nationality)
    .bind(residency)
    .bind(email)
    .bind(phone_number)
    .bind(residential_address)
    .bind(mailing_address)
    .bind(employment_status)
    .bind(occupation)
    .bind(employer_name)
    .bind(monthly_income_range)
    .fetch_one(db)
    .await
}
