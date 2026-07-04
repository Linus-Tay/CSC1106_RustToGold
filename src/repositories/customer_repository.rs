use crate::models::{AccountCreationLink, Customer, KnownDevice, OTPCode, Product};
use chrono::{Duration, NaiveDate, Utc};
use sqlx::{PgPool, Postgres, Transaction as DbTransaction};
use uuid::Uuid;

// Template to store new customer info, this is done as some fields can be left empty, so we use another struct to hold valid data, 'a is used as this struct lifetime only need to exist while creating the customer
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

const CUSTOMER_SELECT: &str = r#"
    SELECT id, full_name, nric, date_of_birth, gender, nationality, residency, race,
           email, phone_number, residential_address, mailing_address, preferred_contact,
           employment_status, occupation, employer_name, industry, monthly_income_range,
           kyc_status, created_at, updated_at
    FROM customers
"#;

// Gets customer using NRIC
pub async fn get_customer_by_nric(
    db: &PgPool,
    nric: &str,
) -> Result<Option<Customer>, sqlx::Error> {
    let query = format!("{} WHERE nric = $1", CUSTOMER_SELECT);
    sqlx::query_as::<_, Customer>(&query)
        .bind(nric)
        .fetch_optional(db)
        .await
}


// Gets non rejected customer using NRIC
pub async fn get_non_rejected_customer_by_nric(
    db: &PgPool,
    nric: &str,
) -> Result<Option<Customer>, sqlx::Error> {
    let query = format!("{} WHERE lower(nric) = lower($1) AND kyc_status <> 'rejected'", CUSTOMER_SELECT);
    sqlx::query_as::<_, Customer>(&query)
        .bind(nric)
        .fetch_optional(db)
        .await
}

// Gets non rejected customer using email
pub async fn get_non_rejected_customer_by_email(
    db: &PgPool,
    email: &str,
) -> Result<Option<Customer>, sqlx::Error> {
    let query = format!("{} WHERE lower(email) = lower($1) AND kyc_status <> 'rejected'", CUSTOMER_SELECT);
    sqlx::query_as::<_, Customer>(&query)
        .bind(email)
        .fetch_optional(db)
        .await
}

// Gets customer using ID
pub async fn get_customer_by_id(db: &PgPool, id: &Uuid) -> Result<Customer, sqlx::Error> {
    let query = format!("{} WHERE id = $1", CUSTOMER_SELECT);
    sqlx::query_as::<_, Customer>(&query)
        .bind(id)
        .fetch_one(db)
        .await
}

// Creates the customer and product
pub async fn create_customer_and_product(
    db: &PgPool,
    new_customer: &NewCustomer<'_>,
    product_id: &str,
    product_type: &str,
    account_number: &str,
) -> Result<(Customer, Product), sqlx::Error> {
    let mut tx: DbTransaction<'_, Postgres> = db.begin().await?;

    let customer = sqlx::query_as::<_, Customer>(
        r#"
        INSERT INTO customers (
            full_name, nric, date_of_birth, gender, nationality, residency, race,
            email, phone_number, residential_address, mailing_address, preferred_contact,
            employment_status, occupation, employer_name, industry, monthly_income_range, kyc_status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, COALESCE($18, 'pending'))
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
    .bind(new_customer.kyc_status)
    .fetch_one(&mut *tx)
    .await?;

    let product = sqlx::query_as::<_, Product>(
        r#"
        INSERT INTO customer_products (customer_id, product_id, product_type, account_number, balance_cents, status)
        VALUES ($1, $2, $3, $4, 0, 'inactive')
        RETURNING id, customer_id, account_number, product_id, product_type, balance_cents, status, created_at, updated_at
        "#,
    )
    .bind(customer.id)
    .bind(product_id)
    .bind(product_type)
    .bind(account_number)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((customer, product))
}

// Updates editable info such as full name and phone number
pub async fn update_basic_profile(
    db: &PgPool,
    customer_id: Uuid,
    full_name: &str,
    phone_number: &str,
) -> Result<Customer, sqlx::Error> {
    sqlx::query_as::<_, Customer>(
        r#"
        UPDATE customers
        SET full_name = $1,
            phone_number = $2,
            updated_at = NOW()
        WHERE id = $3
        RETURNING id, full_name, nric, date_of_birth, gender, nationality, residency, race,
                  email, phone_number, residential_address, mailing_address, preferred_contact,
                  employment_status, occupation, employer_name, industry, monthly_income_range,
                  kyc_status, created_at, updated_at
        "#,
    )
    .bind(full_name)
    .bind(phone_number)
    .bind(customer_id)
    .fetch_one(db)
    .await
}

// Updates the customer and the product linked to them
pub async fn approve_customer_and_product(
    db: &PgPool,
    customer_id: &Uuid,
    product_id: &Uuid,
) -> Result<(Customer, Product), sqlx::Error> {
    let mut tx: DbTransaction<'_, Postgres> = db.begin().await?;

    let updated_customer = sqlx::query_as::<_, Customer>(
        r#"
        UPDATE customers
        SET kyc_status = 'approved', updated_at = NOW()
        WHERE id = $1 AND kyc_status = 'pending'
        RETURNING id, full_name, nric, date_of_birth, gender, nationality, residency, race,
                  email, phone_number, residential_address, mailing_address, preferred_contact,
                  employment_status, occupation, employer_name, industry, monthly_income_range,
                  kyc_status, created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .fetch_one(&mut *tx)
    .await?;

    let updated_product = sqlx::query_as::<_, Product>(
        r#"
        UPDATE customer_products
        SET status = 'active', updated_at = NOW()
        WHERE id = $1 AND status = 'inactive'
        RETURNING id, customer_id, account_number, product_id, product_type, balance_cents, status, created_at, updated_at
        "#,
    )
    .bind(product_id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((updated_customer, updated_product))
}

// Creates the unqiue account creation link so customer can use it to create their online banking user account
pub async fn create_user_account_creation_link_for_customer(
    db: &PgPool,
    customer_id: &Uuid,
) -> Result<AccountCreationLink, sqlx::Error> {
    sqlx::query_as::<_, AccountCreationLink>(
        r#"
        INSERT INTO account_creation_links (customer_id, status, expires_at)
        VALUES ($1, 'pending', $2)
        RETURNING id, customer_id, status, expires_at, created_at
        "#,
    )
    .bind(customer_id)
    .bind(Utc::now() + Duration::hours(72))
    .fetch_one(db)
    .await
}

// Gets the unique account creation link using the link_id
pub async fn get_account_creation_link(
    db: &PgPool,
    account_creation_link: &Uuid,
) -> Result<Option<AccountCreationLink>, sqlx::Error> {
    sqlx::query_as::<_, AccountCreationLink>(
        r#"
        SELECT id, customer_id, status, expires_at, created_at
        FROM account_creation_links
        WHERE id = $1
        "#,
    )
    .bind(account_creation_link)
    .fetch_optional(db)
    .await
}

// Invalidates the account creation link after user uses them
pub async fn invalidate_account_creation_link(
    db: &PgPool,
    account_creation_link: &Uuid,
) -> Result<AccountCreationLink, sqlx::Error> {
    sqlx::query_as::<_, AccountCreationLink>(
        r#"
        UPDATE account_creation_links
        SET status = 'used'
        WHERE id = $1
        RETURNING id, customer_id, status, expires_at, created_at
        "#,
    )
    .bind(account_creation_link)
    .fetch_one(db)
    .await
}

// Gets the device by the hashed device token to check for known devices
pub async fn find_device_by_hashed_token(db: &PgPool, hashed_token: &str) -> Result<Option<KnownDevice>, sqlx::Error> {
    sqlx::query_as::<_, KnownDevice>(r#"
        SELECT id, token_hash, user_id, last_used
        FROM known_devices
        WHERE token_hash = $1
    "#)
    .bind(hashed_token)
    .fetch_optional(db)
    .await
}


// Create a known device
pub async fn create_known_device(
    db: &PgPool,
    user_id: &Uuid,
    hashed_token: &str,
) -> Result<KnownDevice, sqlx::Error> {
     sqlx::query_as::<_, KnownDevice>(
        r#"
        INSERT INTO known_devices (user_id, token_hash)
        VALUES ($1, $2)
        RETURNING id, user_id, token_hash, last_used
        "#,
    )
    .bind(user_id)
    .bind(hashed_token)
    .fetch_one(db)
    .await
}

// Modify the last used timing for a known device
pub async fn update_known_device_last_used(db: &PgPool, id: &Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE known_devices SET last_used = NOW() WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;

    Ok(())
}

// Gets the OTP code obj using the code id
pub async fn get_otp_code(db: &PgPool, code: &str) -> Result<Option<OTPCode>, sqlx::Error> {
    sqlx::query_as::<_, OTPCode>(r#"
        SELECT id, user_id, code, expires_at, created_at
        FROM otp_codes
        WHERE code = $1

    "#)
    .bind(code)
    .fetch_optional(db)
    .await
}

// Deletes otp code once used
pub async fn delete_otp_code(db: &PgPool, id: &Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(r#"
        DELETE FROM otp_codes
        WHERE id = $1
    "#)
        .bind(id)
        .execute(db)
        .await?;

    Ok(())
}

// Creates a new otp code
pub async fn create_otp_code(
    db: &PgPool,
    user_id: &Uuid,
    otp_code: &str,
) -> Result<OTPCode, sqlx::Error> {
     sqlx::query_as::<_, OTPCode>(
        r#"
        INSERT INTO otp_codes (user_id, code, expires_at)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, code, expires_at, created_at
        "#,
    )
    .bind(user_id)
    .bind(otp_code)
    .bind(Utc::now() + Duration::minutes(2))
    .fetch_one(db)
    .await
}