use crate::controllers::onboard_controller::OnboardingFormData;
use crate::forms::DepositForm;
use crate::models::{AccountWorkflow, BankAccount, Customer, Money, Transaction};
use crate::repositories::customer_repository::NewCustomer;
use crate::repositories::{account_repository, customer_repository, transaction_repository};
use crate::services::support::clean_optional_text;
use crate::AppState;
use chrono::NaiveDate;
use sqlx::PgPool;

pub async fn create_customer(db: &PgPool, form: OnboardingFormData) -> Result<Customer, String> {
    let step1 = form.step1.as_ref().ok_or("Missing onboarding step1 data")?;

    println!("{}", step1.nric);

    let customer_option = customer_repository::get_customer_by_nric(db, step1.nric.clone())
        .await
        .map_err(|e| e.to_string())?;

    println!("{}", step1.full_name);
    //println!("{}", customer_option.clone().unwrap().id);

    let new_customer_data = NewCustomer {
        // Other mandatory fields...
        full_name: &step1.full_name.clone(),
        nric: &step1.nric.clone(),
        residency: &step1.residential_status.clone(),
        date_of_birth: NaiveDate::from_ymd_opt(2026, 01, 01).unwrap(),
        gender: &String::from("Male"),
        nationality: &String::from("Singaporean"),
        race: Some(&String::from("Chinese")),
        email: &String::from("test@gmail.com"),
        phone_number: &String::from("911111112"),
        preferred_contact: None,
        mailing_address: Some(&String::from("Random Address")),
        residential_address: &String::from("Random Address"),
        employment_status: &String::from("Unemployed"),
        occupation: None,
        employer_name: None,
        monthly_income_range: None,
        industry: None,
        kyc_status: None,
    };

    let final_customer = match customer_option {
        Some(existing_customer) => {
            println!("some: {}", existing_customer.id);
            customer_repository::update_customer(db, existing_customer.id, &new_customer_data)
                .await
                .map_err(|e| e.to_string())?
        },
        None => {
            println!("None ran");
            customer_repository::create_customer(db, &new_customer_data)
                .await
                .map_err(|e| e.to_string())?
        }
    };

    Ok(final_customer)
}