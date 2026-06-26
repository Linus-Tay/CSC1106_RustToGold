use crate::models::{AccountWorkflow, BankAccount, Customer, Money, Transaction};
use crate::repositories::customer_repository::NewCustomer;
use crate::repositories::{account_repository, customer_repository, transaction_repository};
use crate::services::support::clean_optional_text;
use crate::AppState;
use chrono::NaiveDate;
use sqlx::PgPool;

// pub async fn register_customer(db: &PgPool, form: OnboardingForm) -> Result<Customer, String> {
//     // This replaces your entire first `if let` block
//     let step1 = form.step1.as_ref().ok_or("Missing onboarding step1 data")?;

//     println!("{}", step1.nric);

//     let customer_option = customer_repository::get_customer_by_nric(db, step1.nric.clone())
//         .await
//         .map_err(|e| e.to_string())?;

//     println!("{}", step1.full_name);
//     //println!("{}", customer_option.clone().unwrap().id);

//     let new_customer_data = NewCustomer {
//         // Other mandatory fields...
//         full_name: &step1.full_name.clone(),
//         nric: &step1.nric.clone(),
//         residency: &step1.residential_status.clone(),
//         date_of_birth: NaiveDate::from_ymd_opt(2026, 01, 01).unwrap(),
//         gender: &String::from("Male"),
//         nationality: &String::from("Singaporean"),
//         race: Some(&String::from("Chinese")),
//         email: &String::from("test@gmail.com"),
//         phone_number: &String::from("911111112"),
//         mailing_address: Some(&String::from("Random Address")),
//         residential_address: &String::from("Random Address"),
//         employment_status: &String::from("Unemployed"),
//         preferred_contact: None,
//         occupation: None,
//         employer_name: None,
//         monthly_income_range: None,
//         industry: None,
//         kyc_status: None,
//     };

//     let final_customer = match customer_option {
//         Some(existing_customer) => {
//             println!("some: {}", existing_customer.id);
//             customer_repository::update_customer(db, existing_customer.id, &new_customer_data)
//                 .await
//                 .map_err(|e| e.to_string())?
//         },
//         None => {
//             println!("None ran");
//             customer_repository::create_customer(db, &new_customer_data)
//                 .await
//                 .map_err(|e| e.to_string())?
//         }
//     };

//     Ok(final_customer)
// }

pub async fn load_customer_dashboard(
    db: &PgPool,
    user_id: i64,
) -> Result<(BankAccount, Vec<Transaction>), String> {
    let account = account_repository::find_primary_account_by_user_id(db, user_id)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No bank account was found for this customer.".to_string())?;

    let transactions = transaction_repository::find_recent_transactions_by_user_id(db, user_id, 5)
        .await
        .map_err(|_| "Could not load recent transactions.".to_string())?;

    Ok((account, transactions))
}

pub async fn list_transactions(db: &PgPool, user_id: i64) -> Result<Vec<Transaction>, String> {
    transaction_repository::find_recent_transactions_by_user_id(db, user_id, 50)
        .await
        .map_err(|_| "Could not load transaction history.".to_string())
}

// pub async fn deposit(app_state: &AppState, user_id: i64, form: DepositForm) -> Result<BankAccount, String> {
//     let amount = Money::parse_dollars(&form.amount)?;
//     let description = clean_optional_text(&form.description);
//     let current_account = account_repository::find_primary_account_by_user_id(&app_state.db, user_id)
//         .await
//         .map_err(|_| "Could not load your bank account.".to_string())?
//         .ok_or_else(|| "No bank account was found for this customer.".to_string())?;

//     if !current_account.is_open_for_customer_actions() {
//         return Err("This account is not open for deposits.".to_string());
//     }

//     if current_account.projected_balance_after_deposit(amount).is_none() {
//         return Err("This deposit cannot be applied to the account.".to_string());
//     }

//     let _guard = app_state.account_mutex.lock().await;

//     let (updated_account, _) = account_repository::deposit_to_primary_account(
//         &app_state.db,
//         user_id,
//         amount.cents(),
//         description.as_deref(),
//     )
//     .await
//     .map_err(|_| "Deposit failed. Please try again.".to_string())?;

//     Ok(updated_account)
// }
