use crate::forms::{DepositForm, OnboardingForm};
use crate::models::{AccountWorkflow, BankAccount, Customer, Money, Product, Transaction};
use crate::repositories::customer_repository::NewCustomer;
use crate::repositories::{account_repository, customer_repository, product_repository, transaction_repository};
use crate::services::email_service;
use crate::services::support::clean_optional_text;
use crate::views::templates::ApplicationReceivedTemplate;
use crate::{AppState, services};
use askama::DynTemplate;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_customer_with_product(db: &PgPool, form: &OnboardingForm, product_type: String) -> Result<(Customer, Product), String> {
    let step1 = form.step1.as_ref().ok_or("Missing onboarding step1 data")?;
    let step2 = form.step2.as_ref().ok_or("Missing onboarding step2 data")?;
    let step3 = form.step3.as_ref().ok_or("Missing onboarding step3 data")?;
    let step4 = form.step4.as_ref().ok_or("Missing onboarding step4 data")?;

    let customer_option = customer_repository::get_customer_by_nric(db, &step2.nric.clone())
        .await
        .map_err(|e| e.to_string())?;

    let dob =  NaiveDate::parse_from_str(&step2.dob.clone(), "%Y-%m-%d")
    .map_err(|_| "Date of birth must be in YYYY-MM-DD format")?;


    let new_customer_data = NewCustomer {
        // Other mandatory fields...
        full_name: &step2.full_name,
        nric: &step2.nric.clone(),
        residency: &step2.residential_status,
        date_of_birth: dob,
        gender: &step2.gender,
        nationality: &step2.nationality,
        race: &step2.race,
        email: &step3.email,
        phone_number: &step3.phone_number,
        preferred_contact: Some(&String::from("email")),
        mailing_address: step3.mailing_address.as_deref(),
        residential_address: &step2.residential_address,
        employment_status: &step4.employment_status,
        occupation: step4.occupation.as_deref(),
        employer_name: step4.employer_name.as_deref(),
        monthly_income_range: step4.monthly_income_range.as_deref(),
        industry: step4.industry.as_deref(),
        kyc_status: None,
    };

    let (final_customer, product) = match customer_option {
        Some(existing_customer) => {
            println!("some: {}", existing_customer.id);
            // customer_repository::update_customer(db, existing_customer.id, &new_customer_data)
            //     .await
            //     .map_err(|e| e.to_string())?
            return Err("Customer found".to_string());
        },
        None => {
            println!("None ran");
            let account_number = services::generate_account_number(db).await;
            customer_repository::create_customer_and_product(db, &new_customer_data, &step1.clone().selected_account_type.as_deref().unwrap(), &"savings".to_string(), &account_number)
                .await
                .map_err(|e| e.to_string())?
        }
    };

    let email_to_send = step3.email.clone();
    let subject_to_send = format!("Rust To Gold Application received {}", product.id);
    let template = ApplicationReceivedTemplate {};

    tokio::spawn(async move {
        let result = services::send_template_email(
            &email_to_send, 
            &subject_to_send, 
            &template
        ).await;
        
        if let Err(e) = result {
            println!("Background task failed to send email: {}", e);
        }
    });

    Ok((final_customer, product))
}

pub async fn approve_customer_with_product(app_state: &AppState, customer_id: Uuid) -> Result<(Customer, Product), String> {
    let first_inactive_product = product_repository::get_first_product_by_customer_id(&app_state.db, &customer_id)
    .await
    .map_err(|e| {
        println!("Error while getting first inactive product: {}", e.to_string());
        "No product found to approve".to_string()
    })?;

    let (customer, product) = match customer_repository::approve_customer_and_product(&app_state.db, &customer_id, &first_inactive_product.id).await {
        Ok((customer, product)) => (customer, product),
        Err(e) => {
            println!("Error while approving customer and product: {}", e.to_string());
            return Err("Account approval failed. Please try again later".to_string())
        }
    };

    // let customer = customer_repository::get_customer_by_id(&app_state.db, &pending_product.customer_id)
    //     .await
    //     .map_err(|e| {
    //         println!("error from database: {}", e.to_string());
    //         "An error occurred when retrieving customer data.".to_string()
    //     })?;

    // if customer.kyc_status != "approved" {
    //     customer_repository::approve_customer(&app_state.db, &customer.id)
    //         .await
    //         .map_err(|e| {
    //             println!("error from database: {}", e.to_string());
    //             "KYC approval failed. Please try again later.".to_string()
    //         })?;
    // }

    // let updated_product = product_repository::approve_product(&app_state.db, &account_id)
    //     .await
    //     .map_err(|e| {
    //         println!("error from database: {}", e.to_string());
    //         "Account approval failed. Please try again later.".to_string()
    //     })?;

    Ok((customer, product))
}