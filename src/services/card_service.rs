// Service layer: keeps banking validation and workflow rules away from templates and SQL.

use crate::forms::CardApplicationForm;
use crate::models::{Card, Product};
use crate::repositories::{card_repository, product_repository};
use sqlx::PgPool;
use uuid::Uuid;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

// Data carrier for the CardDashboardData workflow.
pub struct CardDashboardData {
    pub cards: Vec<Card>,
    pub has_cards: bool,
    pub accounts: Vec<Product>,
    pub has_accounts: bool,
}

// Loads card dashboard data and applies page-level business rules.
pub async fn load_card_dashboard(db: &PgPool, customer_id: Uuid) -> Result<CardDashboardData, String> {
    let cards = card_repository::list_cards_by_customer(db, customer_id)
        .await
        .map_err(|e| {
            println!("error: {}", e.to_string());
            "Could not load your cards.".to_string()
        })?;
    let accounts = product_repository::list_active_products_by_customer(db, &customer_id)
        .await
        .map_err(|_| "Could not load linkable accounts.".to_string())?;

    Ok(CardDashboardData {
        has_cards: !cards.is_empty(),
        has_accounts: !accounts.is_empty(),
        cards,
        accounts,
    })
}

// Validates and coordinates the create card workflow.
pub async fn create_card(db: &PgPool, customer_id: Uuid, form: CardApplicationForm) -> Result<Card, String> {
    let linked_product_id = Uuid::parse_str(form.linked_account_id.trim())
        .map_err(|_| "Choose a valid account to link this card to.".to_string())?;

    let pin_number = form.pin_number;

    if pin_number.len() != 4 {
        return Err("Invalid pin number".to_string());
    }

    let hashed_pin = hash_pin(&pin_number).expect("Failed to hash pin");

    let card_type = match form.card_type.trim() {
        "debit" => "debit",
        "student" => "student",
        _ => return Err("Choose a valid card type.".to_string()),
    };

    let account = product_repository::get_active_product_for_customer_by_id(db, customer_id, linked_product_id)
        .await
        .map_err(|_| "The selected account is not active or does not belong to you.".to_string())?;

    if let Ok(Some(existing_linked_card)) = card_repository::find_card_by_linked_account(db, linked_product_id).await {
        return Err("You already have an existing card".to_string());
    }

    let display_name = form
        .display_name
        .unwrap_or_default()
        .trim()
        .to_string();
    let display_name = if display_name.is_empty() {
        match card_type {
            "student" => "Campus Student Card".to_string(),
            _ => "Everyday Debit Card".to_string(),
        }
    } else {
        display_name
    };

    let card_number = generate_card_from_account(&account.account_number);

    card_repository::create_card(db, customer_id, linked_product_id, card_type, &hashed_pin, &display_name, &card_number)
        .await
        .map_err(|e| {
            println!("error creating card: {}", e.to_string());
            "Could not create the card.".to_string()
        })
}

// Validates and coordinates the set card status workflow.
pub async fn set_card_status(db: &PgPool, customer_id: Uuid, card_id: Uuid, status: &str) -> Result<(), String> {
    let status = match status {
        "active" => "active",
        "frozen" => "frozen",
        "cancelled" => "cancelled",
        _ => return Err("Invalid card status.".to_string()),
    };
    card_repository::set_card_status(db, customer_id, card_id, status)
        .await
        .map_err(|_| "Could not update card status.".to_string())
}

pub async fn validate_card(db: &PgPool, card_number: &str) -> Result<Option<Card>, String> {
    let card = card_repository::find_active_by_card_number(db, card_number)
    .await
    .map_err(|_| "Invalid card".to_string())?;

    Ok(card)
}

pub async fn authenticate_card(db: &PgPool, card_number: &str, pin: &str) -> Result<Card, String> {
    let card = card_repository::find_active_by_card_number(db, card_number)
        .await
        .map_err(|_| "Invalid card".to_string())?
        .ok_or_else(|| "No card found".to_string())?;

    match verify_pin(pin, &card.pin_hash) {
        true => Ok(card),
        _ => {
            return Err("Invalid pin".to_string())
        }
    }
}

/// Hashes the raw PIN (e.g., "1234") into a secure string.
pub fn hash_pin(pin: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(pin.as_bytes(), &salt)
        .map_err(|e| e.to_string())?;

    Ok(password_hash.to_string())
}

/// Verifies a raw PIN against the hashed string stored in your database.
pub fn verify_pin(raw_pin: &str, stored_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(stored_hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(raw_pin.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn generate_card_from_account(account_number: &str) -> String {
    let bin = "400000"; 
    
    let clean_account: String = account_number
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();

    let truncated_account = &clean_account[clean_account.len().saturating_sub(9)..];
        
    let partial_pan = format!("{}{}", bin, truncated_account);
    
    let check_digit = calculate_luhn(&partial_pan);
    format!("{}{}", partial_pan, check_digit)
}

fn calculate_luhn(partial_pan: &str) -> u32 {
    let mut sum = 0;
    let mut double = true; 

    for c in partial_pan.chars().rev() {
        if let Some(mut digit) = c.to_digit(10) {
            if double {
                digit *= 2;
                if digit > 9 {
                    digit -= 9;
                }
            }
            sum += digit;
            double = !double;
        }
    }
    
    (10 - (sum % 10)) % 10
}