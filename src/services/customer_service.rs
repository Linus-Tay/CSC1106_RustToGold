use crate::forms::OnboardingForm;
use crate::models::{Customer, Product};
use crate::repositories::customer_repository::NewCustomer;
use crate::repositories::{customer_repository, product_repository};
use crate::views::templates::{AccountCreationEmailTemplate, ApplicationReceivedEmailTemplate};
use crate::{services, AppState};
use askama::DynTemplate;
use chrono::NaiveDate;
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

// Ensure the onboarding url is clean and valid, otherwise redirect to main page
fn onboarding_base_url() -> String {
    env::var("ONBOARDING_BASE_URL").or_else(|_| env::var("APP_BASE_URL"))
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:3000".to_string())
}

// Calls the repo to create a customer with linked product info
pub async fn create_customer_with_product(
    db: &PgPool,
    form: &OnboardingForm,
    product_type: String,
) -> Result<(Customer, Product), String> {
    let step1 = form.step1.as_ref().ok_or("Missing account selection data")?;
    let step2 = form.step2.as_ref().ok_or("Missing personal details")?;
    let step3 = form.step3.as_ref().ok_or("Missing contact details")?;
    let step4 = form.step4.as_ref().ok_or("Missing employment details")?;

    if customer_repository::get_non_rejected_customer_by_nric(db, &step2.nric)
        .await
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Err("We are unable to perform this action at this moment. Please try again later.".to_string());
    }

    let dob = NaiveDate::parse_from_str(&step2.dob, "%Y-%m-%d")
        .map_err(|_| "Date of birth must be in YYYY-MM-DD format".to_string())?;

    let new_customer_data = NewCustomer {
        full_name: &step2.full_name,
        nric: &step2.nric,
        residency: &step2.residential_status,
        date_of_birth: dob,
        gender: &step2.gender,
        nationality: &step2.nationality,
        race: Some(&step2.race),
        email: &step3.email,
        phone_number: &step3.phone_number,
        preferred_contact: Some("email"),
        mailing_address: step3.mailing_address.as_deref(),
        residential_address: &step2.residential_address,
        employment_status: &step4.employment_status,
        occupation: step4.occupation.as_deref(),
        employer_name: step4.employer_name.as_deref(),
        monthly_income_range: step4.monthly_income_range.as_deref(),
        industry: step4.industry.as_deref(),
        kyc_status: Some("pending"),
    };

    let account_number = services::generate_account_number(db).await;
    let product_id = step1
        .selected_account_type
        .as_deref()
        .unwrap_or("everyday_savings");

    let result = customer_repository::create_customer_and_product(
        db,
        &new_customer_data,
        product_id,
        &product_type,
        &account_number,
    )
    .await
    .map_err(|error| {
        eprintln!("CUSTOMER application insert failed: {error:?}");
        "Could not submit your account application. Please check the server log for details."
            .to_string()
    })?;

    let email_to_send = step3.email.clone();
    let subject_to_send = format!("RustToGold application received {}", result.1.id);
    let template = ApplicationReceivedEmailTemplate {};

    println!(
        "APPLICATION EMAIL: queueing application-received email to {email_to_send}. This is not the activation email."
    );

    match template.dyn_render() {
        Ok(html_body) => {
            tokio::spawn(async move {
                if let Err(error) = services::send_html_email(&email_to_send, &subject_to_send, html_body).await {
                    eprintln!("Application received email failed: {error}");
                }
            });
        }
        Err(error) => eprintln!("Application received email render failed: {error}"),
    }

    Ok(result)
}

// Approve customer and the linked product
pub async fn approve_customer_with_product(
    app_state: &AppState,
    customer_id: Uuid,
) -> Result<(Customer, Product), String> {
    let first_inactive_product = product_repository::get_first_product_by_customer_id(
        &app_state.db,
        &customer_id,
    )
    .await
    .map_err(|error| {
        eprintln!("Could not load pending product for customer {customer_id}: {error:?}");
        "No pending product found to approve.".to_string()
    })?;

    let (customer, product) = customer_repository::approve_customer_and_product(
        &app_state.db,
        &customer_id,
        &first_inactive_product.id,
    )
    .await
    .map_err(|error| {
        eprintln!("Account approval failed for customer {customer_id}: {error:?}");
        "Account approval failed. Please try again later.".to_string()
    })?;

    let account_creation_link = customer_repository::create_user_account_creation_link_for_customer(
        &app_state.db,
        &customer_id,
    )
    .await
    .map_err(|error| {
        eprintln!("Account creation link failed for customer {customer_id}: {error:?}");
        "Account was approved, but the account creation email could not be prepared.".to_string()
    })?;

    let activation_url = format!(
        "{}/account-creation/init?link={}",
        onboarding_base_url(),
        account_creation_link.get_link()
    );

    println!("ACCOUNT ACTIVATION LINK: {activation_url}");

    let email_to_send = customer.email.clone();
    let subject_to_send = format!(
        "Welcome to RustToGold - set up online banking for {}",
        product.account_number
    );
    let template = AccountCreationEmailTemplate {
        account_creation_link: activation_url,
    };

    println!(
        "ACTIVATION EMAIL: sending account-creation email to {email_to_send} after admin approval."
    );

    if let Err(error) = services::send_template_email(&email_to_send, &subject_to_send, &template).await {
        eprintln!("Account creation email failed: {error}");
    }

    Ok((customer, product))
}

// Calls repo to check if the account creation link is valid
pub async fn validate_account_creation_link(
    app_state: &AppState,
    account_creation_link: &str,
) -> Result<bool, String> {
    let link_uuid = Uuid::parse_str(account_creation_link)
        .map_err(|_| "Failed to parse account creation link.".to_string())?;

    let Some(link) = customer_repository::get_account_creation_link(&app_state.db, &link_uuid)
        .await
        .map_err(|error| {
            eprintln!("Account creation link lookup failed: {error:?}");
            "Invalid account creation link.".to_string()
        })?
    else {
        return Ok(false);
    };

    Ok(link.is_valid())
}

// Gets the customer info using account creation link
pub async fn get_customer_by_account_creation_link(
    app_state: &AppState,
    account_creation_link: &str,
) -> Result<Customer, String> {
    let link_uuid = Uuid::parse_str(account_creation_link)
        .map_err(|_| "Failed to parse account creation link.".to_string())?;

    let link = customer_repository::get_account_creation_link(&app_state.db, &link_uuid)
        .await
        .map_err(|error| {
            eprintln!("Account creation link lookup failed: {error:?}");
            "Invalid account creation link.".to_string()
        })?
        .ok_or_else(|| "Account creation link not found.".to_string())?;

    if !link.is_valid() {
        return Err("Account creation link is invalid or expired.".to_string());
    }

    customer_repository::get_customer_by_id(&app_state.db, &link.get_customer_id())
        .await
        .map_err(|error| {
            eprintln!("Customer lookup from account creation link failed: {error:?}");
            "Could not load customer for this link.".to_string()
        })
}

// Calls the repo to invalidate the account creation link once used
pub async fn invalidate_account_creation_link(
    app_state: &AppState,
    account_creation_link: &str,
) -> Result<bool, String> {
    let link_uuid = Uuid::parse_str(account_creation_link)
        .map_err(|_| "Failed to parse account creation link.".to_string())?;

    customer_repository::invalidate_account_creation_link(&app_state.db, &link_uuid)
        .await
        .map(|_| true)
        .map_err(|error| {
            eprintln!("Account creation link invalidation failed: {error:?}");
            "Could not mark account creation link as used.".to_string()
        })
}
