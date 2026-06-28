use crate::forms::DepositForm;
use crate::forms::account_forms::TransferForm;
use crate::models::product::ProductWorkflow;
use crate::models::{AccountWorkflow, BankAccount, Customer, Money, Product, Transaction};
use crate::repositories::customer_repository::NewCustomer;
use crate::repositories::{account_repository, customer_repository, product_repository, transaction_repository};
use crate::services::email_service;
use crate::services::support::clean_optional_text;
use crate::AppState;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use rand::{rng, RngExt};

pub async fn create_product(db: &PgPool, customer_id: Uuid, product_id: String, product_type: String ) -> Result<Product, String> {
    println!("this ran??");
    let product_option = product_repository::get_product_by_user_id_and_product_id(db, &customer_id, &product_id)
    .await
    .map_err(|e| e.to_string())?;

    if product_option.is_some() {
        println!("caught here xd");
        return Err("lol".to_string());
    }

    let account_number = generate_account_number(db)
    .await;

    let product = product_repository::insert_product(db, &customer_id, &product_id, &product_type, &account_number)
    .await;

    match product {
        Ok(product) => Ok(product),
        Err(err_message) => return Err(err_message.to_string()),
    }
}

pub async fn deposit(app_state: &AppState, customer_id: Uuid, form: DepositForm) -> Result<Product, String> {
    let amount = Money::parse_dollars(&form.amount)?;
    let description = clean_optional_text(&form.description);
    let account_number = &form.account_number;
    let current_product = product_repository::get_product_by_account_number(&app_state.db, &account_number)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No bank account was found under this number.".to_string())?;

    if current_product.get_customer_id() != customer_id {
        return Err("You cannot deposit to accounts that is not owned by you".to_string());
    }

    if !current_product.is_open_for_customer_actions() {
        return Err("This account is not open for deposits.".to_string());
    }

    if current_product.projected_balance_after_deposit(amount).is_none() {
        return Err("This deposit cannot be applied to the account.".to_string());
    }

    //let _guard = app_state.account_mutex.lock().await;

    let (updated_product, _) = product_repository::deposit_into_product(&app_state.db, &customer_id, account_number, amount.cents(), description.as_deref()).await
    .map_err(|e| {
        println!("error from database: {}", e.to_string());
        "Deposit failed. Please try again later.".to_string()
    })?;

    Ok(updated_product)
}

pub async fn transfer(app_state: &AppState, customer_id: Uuid, form: TransferForm) -> Result<bool, String> {
    println!("transfer ran");
    let amount = Money::parse_dollars(&form.amount)?;
    let note = clean_optional_text(&form.note);
    let account_number = &form.account_number;
    let transfer_method = &form.method;
    let recipient_info = &form.recipient_info;
    let sender_product = product_repository::get_product_by_account_number(&app_state.db, &account_number)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No bank account was found under this number.".to_string())?;

    let recipient_product: Product = match transfer_method.as_str() {
        "local" => product_repository::get_product_by_account_number(&app_state.db, recipient_info)
            .await
            .map_err(|_| "Could not load recipient bank account.".to_string())?
            .ok_or_else(|| "No bank account was found under this account number.".to_string())?,
        _ => return Err("Invalid transfer method".to_string())
    };

    if sender_product.customer_id != customer_id {
        return Err("You cannot perform this action".to_string());
    }

    if sender_product.account_number == recipient_product.account_number {
        return Err("You cannot transfer to the same bank account".to_string());
    }

    if !sender_product.is_open_for_customer_actions() || !recipient_product.is_open_for_customer_actions() {
        return Err("You cannot perform this action".to_string());
    }

    match product_repository::transfer(&app_state.db, &account_number, &customer_id, &recipient_product.customer_id, &recipient_product.account_number, amount.cents(), note.as_deref()).await {
        Ok((true, _)) => Ok(true),
        Ok((false, Some(err_msg))) => Err(err_msg),
        Ok((false, None)) => Err("Transfer failed due to an unknown rule.".to_string()),
        Err(e) => {
            println!("error from database: {}", e.to_string());
            Err("A fatal database error occurred.".to_string())
        },
    }
}

pub async fn approve_product(app_state: &AppState, account_id: Uuid) -> Result<Product, String> {
    let pending_product = product_repository::get_product_by_account_id(&app_state.db, &account_id)
        .await
        .map_err(|e| {
            println!("error from database: {}", e.to_string());
            "An error occurred when retrieving the account.".to_string()
        })?;

    let customer = customer_repository::get_customer_by_id(&app_state.db, &pending_product.customer_id)
        .await
        .map_err(|e| {
            println!("error from database: {}", e.to_string());
            "An error occurred when retrieving customer data.".to_string()
        })?;

    if customer.kyc_status != "approved" {
        customer_repository::approve_customer(&app_state.db, &customer.id)
            .await
            .map_err(|e| {
                println!("error from database: {}", e.to_string());
                "KYC approval failed. Please try again later.".to_string()
            })?;
    }

    let updated_product = product_repository::approve_product(&app_state.db, &account_id)
        .await
        .map_err(|e| {
            println!("error from database: {}", e.to_string());
            "Account approval failed. Please try again later.".to_string()
        })?;

    Ok(updated_product)
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


pub async fn generate_account_number(db: &PgPool) -> String {
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
