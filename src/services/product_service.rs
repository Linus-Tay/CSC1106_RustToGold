use crate::controllers::account_controller::OnboardingFormData;
use crate::forms::DepositForm;
use crate::models::customer::{ContactMethod, EmploymentType, Gender, KycStatus, Residency};
use crate::models::{AccountWorkflow, BankAccount, Customer, Money, Product, Transaction};
use crate::repositories::customer_repository::NewCustomer;
use crate::repositories::{account_repository, customer_repository, product_repository, transaction_repository};
use crate::services::support::clean_optional_text;
use crate::AppState;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use rand::{rng, RngExt};

pub async fn create_product(db: &PgPool, customer_id: Uuid, product_id: String) -> Result<Product, String> {
    println!("this ran??");
    let product_option = product_repository::get_product_by_user_id(db, &customer_id, &product_id)
    .await
    .map_err(|e| e.to_string())?;

    if product_option.is_some() {
        println!("caught here xd");
        return Err("lol".to_string());
    }

    let account_number = generate_account_number(db)
    .await;

    let product = product_repository::insert_product(db, customer_id, product_id, account_number)
    .await;

    match product {
        Ok(product) => Ok(product),
        Err(err_message) => return Err(err_message.to_string()),
    }
}

fn luhn_check_digit(number: &str) -> u32 {
    let sum: u32 = number
        .chars()
        .rev()
        .enumerate()
        .map(|(i, c)| {
            let mut digit = c.to_digit(10).unwrap();
            if i % 2 == 0 {
                digit *= 2;
                if digit > 9 {
                    digit -= 9;
                }
            }
            digit
        })
        .sum();

    (10 - (sum % 10)) % 10
}


async fn generate_account_number(db: &PgPool) -> String {
    let mut rng: rand::prelude::ThreadRng = rng();
    let prefix = "7282";

    loop {
        let random_part: String = (0..7)
        .map(|_| rng.random_range(0..10).to_string())
        .collect();
        
        let base = format!("{}{}", prefix, random_part);
        let check_digit = luhn_check_digit(&base);
        
        let full = format!("{}{}", base, check_digit);

        let account_number = format!("{}-{}-{}", &full[0..4], &full[4..11], &full[11..12]);

        let product_option = product_repository::get_product_by_account_number(db, &account_number)
        .await;

        match product_option {
            Ok(None) => return account_number,
            _ => continue,
        }
    }
}

pub async fn register_customer(db: &PgPool, form: OnboardingFormData) -> Result<Customer, String> {
    // This replaces your entire first `if let` block
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
        residency: step1.residential_status.clone(),
        date_of_birth: NaiveDate::from_ymd_opt(2026, 01, 01).unwrap(),
        gender: Gender::Male,
        nationality: &String::from("Singaporean"),
        race: Some(&String::from("Chinese")),
        email: &String::from("test@gmail.com"),
        phone_number: &String::from("911111112"),
        preferred_contact: Some(ContactMethod::Email),
        mailing_address: Some(&String::from("Random Address")),
        residential_address: &String::from("Random Address"),
        employment_status: EmploymentType::Unemployed,
        occupation: None,
        employer_name: None,
        monthly_income_range: None,
        industry: None,
        kyc_status: Some(KycStatus::PENDING),
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