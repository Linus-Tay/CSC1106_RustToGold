use crate::forms::{DepositForm, OnboardingForm};
use crate::models::{AccountWorkflow, BankAccount, Customer, Money, Product, Transaction, account_creation_link};
use crate::repositories::customer_repository::NewCustomer;
use crate::repositories::{account_repository, customer_repository, product_repository, transaction_repository};
use crate::services::email_service;
use crate::services::support::clean_optional_text;
use crate::views::templates::{AccountCreationEmailTemplate, ApplicationReceivedEmailTemplate};
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
    let template = ApplicationReceivedEmailTemplate {};

    tokio::spawn(async move {
        let result = services::send_template_email(
            &email_to_send, 
            &subject_to_send, 
            &template
        ).await;
        
        if let Err(e) = result {
            println!("Background task failed to send application received email: {}", e);
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

    let account_creation_link = customer_repository::create_user_account_creation_link_for_customer(&app_state.db, &customer_id)
    .await
    .map_err(|e| {
        println!("Error while creating account creation link: {}", e.to_string());
        "Failed to send account creation email".to_string()
    })?;



    let email_to_send = customer.email.clone();
    let subject_to_send = format!("Welcome to Rust To Gold, your application has been activated {}", product.id);
    let template = AccountCreationEmailTemplate {
        account_creation_link: format!("http://apply.localhost:3000/account-creation?link={}", account_creation_link.get_link())
    };

    tokio::spawn(async move {
        let result = services::send_template_email(
            &email_to_send, 
            &subject_to_send, 
            &template
        ).await;
        
        if let Err(e) = result {
            println!("Background task failed to send account creation email: {}", e);
        }
    });

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

pub async fn validate_account_creation_link(app_state: &AppState, account_creation_link: &str) -> Result<bool, String> {
    let link_uuid = Uuid::parse_str(account_creation_link).map_err(|e| "Failed to parse UUID".to_string())?;
    let account_creation_link = match customer_repository::get_account_creation_link(&app_state.db, &link_uuid).await {
        Ok(Some(account_creation_link)) => account_creation_link,
        Ok(None) => {
            return Ok(false);
        },
        Err(e) => {
            println!("Error getting acocunt creation link: {}", e.to_string());
            return Err("Invalid account creation link".to_string());
        }
    };

    if account_creation_link.is_valid() == false {
        return Ok(false);
    }

    Ok(true)
}

pub async fn get_customer_by_account_creation_link(app_state: &AppState, account_creation_link: &str) -> Result<Customer, String> {
    let link_uuid = Uuid::parse_str(account_creation_link).map_err(|e| "Failed to parse UUID".to_string())?;
    let account_creation_link = match customer_repository::get_account_creation_link(&app_state.db, &link_uuid).await {
        Ok(Some(account_creation_link)) => account_creation_link,
        Ok(None) => {
            return Err("Error getting account creation link".to_string());
        },
        Err(e) => {
            println!("Error getting acocunt creation link: {}", e.to_string());
            return Err("Invalid account creation link".to_string());
        }
    };

    if account_creation_link.is_valid() == false {
        return Err("Invalid or expired link".to_string());
    }

    let customer = customer_repository::get_customer_by_id(&app_state.db, &account_creation_link.get_customer_id())
    .await
    .map_err(|e| {
        println!("Error getting customer: {}", e.to_string());
        "Error getting customer".to_string()
    })?;

    Ok(customer)
}

pub async fn invalidate_account_creation_link(app_state: &AppState, account_creation_link: &str) -> Result<bool, String> {
    let link_uuid = Uuid::parse_str(account_creation_link).map_err(|e| "Failed to parse UUID".to_string())?;
    match customer_repository::invalidate_account_creation_link(&app_state.db, &link_uuid).await {
        Ok(account_creation_link) => Ok(true),
        Err(e) => Ok(false)
    }
}