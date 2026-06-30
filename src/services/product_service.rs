use crate::forms::account_forms::TransferForm;
use crate::forms::DepositForm;
use crate::models::product::ProductWorkflow;
use crate::models::{Money, Product};
use crate::repositories::{customer_repository, product_repository};
use crate::services::support::clean_optional_text;
use crate::AppState;
use rand::{rng, RngExt};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_product(
    db: &PgPool,
    customer_id: Uuid,
    product_id: String,
    product_type: String,
) -> Result<Product, String> {
    let existing_product = product_repository::get_product_by_user_id_and_product_id(
        db,
        &customer_id,
        &product_id,
    )
    .await
    .map_err(|error| error.to_string())?;

    if existing_product.is_some() {
        return Err("This product already exists for this customer.".to_string());
    }

    let account_number = generate_account_number(db).await;

    product_repository::insert_product(db, &customer_id, &product_id, &product_type, &account_number)
        .await
        .map_err(|error| error.to_string())
}


pub async fn list_customer_products(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<Vec<Product>, String> {
    product_repository::list_products_by_customer(db, &customer_id)
        .await
        .map_err(|_| "Could not load your bank accounts.".to_string())
}

pub async fn list_active_customer_products(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<Vec<Product>, String> {
    product_repository::list_active_products_by_customer(db, &customer_id)
        .await
        .map_err(|_| "Could not load your active bank accounts.".to_string())
}

pub async fn create_bank_account(
    db: &PgPool,
    customer_id: Uuid,
    account_type: &str,
) -> Result<Product, String> {
    let active_accounts = product_repository::list_active_products_by_customer(db, &customer_id)
        .await
        .map_err(|_| "Could not check your existing bank accounts.".to_string())?;

    if active_accounts.len() >= 5 {
        return Err("A customer can hold up to 5 active accounts in this simulation.".to_string());
    }

    let (product_id, product_type) = match account_type {
        "high_yield_savings" => ("high_yield_savings", "savings"),
        "spending_account" => ("spending_account", "spending"),
        _ => ("everyday_savings", "savings"),
    };

    let account_number = generate_account_number(db).await;

    product_repository::insert_active_product(
        db,
        &customer_id,
        product_id,
        product_type,
        &account_number,
    )
    .await
    .map_err(|error| {
        eprintln!("create bank account failed: {error:?}");
        "Could not create the new bank account.".to_string()
    })
}

pub async fn load_active_product_by_account_number(
    db: &PgPool,
    customer_id: Uuid,
    account_number: &str,
) -> Result<Product, String> {
    product_repository::get_active_product_for_customer_by_account_number(db, &customer_id, account_number)
        .await
        .map_err(|_| "Selected account is not active or does not belong to you.".to_string())
}

pub async fn deposit(
    app_state: &AppState,
    customer_id: Uuid,
    form: DepositForm,
) -> Result<Product, String> {
    let amount = Money::parse_dollars(&form.amount)?;
    if amount.cents() > 1_000_000_00 {
        return Err("Single customer deposits are capped at $1,000,000.00 for this demo environment.".to_string());
    }

    let description = clean_optional_text(&form.description);
    let account_number = &form.account_number;

    let current_product = product_repository::get_product_by_account_number(&app_state.db, account_number)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No bank account was found under this number.".to_string())?;

    if current_product.get_customer_id() != customer_id {
        return Err("You cannot deposit to an account that is not owned by you.".to_string());
    }

    if !current_product.is_open_for_customer_actions() {
        return Err("This account is not open for deposits.".to_string());
    }

    if current_product.projected_balance_after_deposit(amount).is_none() {
        return Err("This deposit cannot be applied to the account.".to_string());
    }

    let (updated_product, _) = product_repository::deposit_into_product(
        &app_state.db,
        &customer_id,
        account_number,
        amount.cents(),
        description.as_deref(),
    )
    .await
    .map_err(|_| "Deposit failed. Please try again later.".to_string())?;

    Ok(updated_product)
}

pub async fn transfer(
    app_state: &AppState,
    customer_id: Uuid,
    form: TransferForm,
) -> Result<bool, String> {
    let amount = Money::parse_dollars(&form.amount)?;
    let note = clean_optional_text(&form.note);
    let account_number = &form.account_number;
    let transfer_method = &form.method;
    let recipient_info = &form.recipient_info;

    let sender_product = product_repository::get_product_by_account_number(&app_state.db, account_number)
        .await
        .map_err(|_| "Could not load your bank account.".to_string())?
        .ok_or_else(|| "No bank account was found under this number.".to_string())?;

    let recipient_product: Product = match transfer_method.as_str() {
        "local" => product_repository::get_product_by_account_number(&app_state.db, recipient_info)
            .await
            .map_err(|_| "Could not load recipient bank account.".to_string())?
            .ok_or_else(|| "No bank account was found under this account number.".to_string())?,
        _ => return Err("Invalid transfer method.".to_string()),
    };

    if sender_product.customer_id != customer_id {
        return Err("You cannot perform this action.".to_string());
    }

    if sender_product.account_number == recipient_product.account_number {
        return Err("You cannot transfer to the same bank account.".to_string());
    }

    if !sender_product.is_open_for_customer_actions()
        || !recipient_product.is_open_for_customer_actions()
    {
        return Err("Both accounts must be active before a transfer can be made.".to_string());
    }

    match product_repository::transfer(
        &app_state.db,
        account_number,
        &customer_id,
        &recipient_product.customer_id,
        &recipient_product.account_number,
        amount.cents(),
        note.as_deref(),
    )
    .await
    {
        Ok((true, _)) => Ok(true),
        Ok((false, Some(message))) => Err(message),
        Ok((false, None)) => Err("Transfer failed due to an unknown rule.".to_string()),
        Err(_) => Err("A database error occurred while processing the transfer.".to_string()),
    }
}

pub async fn approve_product(app_state: &AppState, account_id: Uuid) -> Result<Product, String> {
    let pending_product = product_repository::get_product_by_account_id(&app_state.db, &account_id)
        .await
        .map_err(|_| "An error occurred when retrieving the account.".to_string())?;

    let customer = customer_repository::get_customer_by_id(&app_state.db, &pending_product.customer_id)
        .await
        .map_err(|_| "An error occurred when retrieving customer data.".to_string())?;

    if customer.kyc_status != "approved" {
        customer_repository::approve_customer(&app_state.db, &customer.id)
            .await
            .map_err(|_| "KYC approval failed. Please try again later.".to_string())?;
    }

    product_repository::approve_product(&app_state.db, &account_id)
        .await
        .map_err(|_| "Account approval failed. Please try again later.".to_string())
}

fn luhn_check_digit(number: &str) -> u32 {
    let sum: u32 = number
        .chars()
        .rev()
        .enumerate()
        .map(|(index, character)| {
            let mut digit = character.to_digit(10).unwrap_or(0);

            if index % 2 == 0 {
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
    let mut rng = rng();
    let prefix = "7282";

    loop {
        let random_part: String = (0..7)
            .map(|_| rng.random_range(0..10).to_string())
            .collect();

        let base = format!("{}{}", prefix, random_part);
        let check_digit = luhn_check_digit(&base);
        let full = format!("{}{}", base, check_digit);
        let account_number = format!("{}-{}-{}", &full[0..4], &full[4..11], &full[11..12]);

        match product_repository::get_product_by_account_number(db, &account_number).await {
            Ok(None) => return account_number,
            _ => continue,
        }
    }
}
