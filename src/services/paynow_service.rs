use crate::forms::{PayNowRegisterForm, PayNowTransferForm};
use crate::models::{Money, PayNowRegistration, Product};
use crate::repositories::{paynow_repository, product_repository};
use crate::services::support::clean_optional_text;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PayNowDashboard {
    pub accounts: Vec<Product>,
    pub registrations: Vec<PayNowRegistration>,
}

pub async fn load_paynow_dashboard(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<PayNowDashboard, String> {
    let accounts = product_repository::list_active_products_by_customer(db, &customer_id)
        .await
        .map_err(|_| "Could not load your active bank accounts.".to_string())?;

    let registrations = paynow_repository::list_by_customer(db, customer_id)
        .await
        .map_err(|_| "Could not load your PayNow registrations.".to_string())?;

    Ok(PayNowDashboard {
        accounts,
        registrations,
    })
}

pub async fn register_paynow(
    db: &PgPool,
    customer_id: Uuid,
    form: PayNowRegisterForm,
) -> Result<(), String> {
    let paynow_type = normalise_paynow_type(&form.paynow_type)?;
    let paynow_id = normalise_paynow_identifier(&paynow_type, &form.paynow_id)?;
    let linked_product_id = Uuid::parse_str(form.linked_product_id.trim())
        .map_err(|_| "Choose a valid account to link.".to_string())?;

    // Only active accounts can receive PayNow funds.
    product_repository::get_active_product_for_customer_by_id(db, customer_id, linked_product_id)
        .await
        .map_err(|_| "Choose an active account that belongs to you.".to_string())?;

    // A PayNow number/NRIC can only point to one active customer at a time.
    if let Some(existing) = paynow_repository::find_active_by_identifier(db, &paynow_type, &paynow_id)
        .await
        .map_err(|_| "Could not check existing PayNow registrations.".to_string())?
    {
        if existing.customer_id != customer_id {
            return Err("This PayNow ID is already registered.".to_string());
        }
    }

    if paynow_type == "phone_number" {
        paynow_repository::upsert_phone_registration(db, customer_id, &paynow_id, linked_product_id)
            .await
            .map_err(|error| {
                eprintln!("PayNow registration update failed: {error:?}");
                "Could not save this PayNow number. Please check the details and try again.".to_string()
            })?;
    } else {
        paynow_repository::insert_registration(
            db,
            customer_id,
            &paynow_type,
            &paynow_id,
            linked_product_id,
        )
        .await
        .map_err(|error| {
            eprintln!("PayNow registration failed: {error:?}");
            "Could not register this PayNow ID. Please check the details and try again.".to_string()
        })?;
    }

    Ok(())
}

pub async fn transfer_paynow(
    db: &PgPool,
    customer_id: Uuid,
    form: PayNowTransferForm,
) -> Result<(), String> {
    let from_product_id = Uuid::parse_str(form.from_product_id.trim())
        .map_err(|_| "Choose a valid account to transfer from.".to_string())?;
    let recipient_type = normalise_paynow_type(&form.recipient_type)?;
    let recipient_id = normalise_paynow_identifier(&recipient_type, &form.recipient_id)?;
    let amount = Money::parse_dollars(&form.amount)?;
    let note = clean_optional_text(&form.note);

    // Keep PayNow single-transfer value inside the configured limit.
    if amount.cents() > 50_000_00 {
        return Err("PayNow transfers are capped at $50,000.00 per transaction.".to_string());
    }

    // Apply daily limit, Money Lock and monitoring before funds move.
    crate::services::transaction_control_service::validate_outgoing_transaction(
        db,
        customer_id,
        Some(from_product_id),
        amount.cents(),
        note.as_deref(),
        "PayNow",
        true,
    )
    .await?;

    match paynow_repository::execute_paynow_transfer(
        db,
        customer_id,
        from_product_id,
        &recipient_type,
        &recipient_id,
        amount.cents(),
        note.as_deref(),
    )
    .await
    {
        Ok((true, _)) => Ok(()),
        Ok((false, Some(message))) => Err(message),
        Ok((false, None)) => Err("PayNow transfer could not be completed.".to_string()),
        Err(error) => {
            eprintln!("PayNow transfer failed: {error:?}");
            Err("A database error occurred while processing the PayNow transfer.".to_string())
        }
    }
}

fn normalise_paynow_type(input: &str) -> Result<String, String> {
    match input.trim() {
        "phone_number" => Ok("phone_number".to_string()),
        "nric" => Ok("nric".to_string()),
        _ => Err("Choose a valid PayNow type.".to_string()),
    }
}

fn normalise_paynow_identifier(paynow_type: &str, input: &str) -> Result<String, String> {
    match paynow_type {
        "phone_number" => normalise_phone_number(input),
        "nric" => normalise_nric(input),
        _ => Err("Choose a valid PayNow type.".to_string()),
    }
}

fn normalise_phone_number(input: &str) -> Result<String, String> {
    let mut value = input
        .trim()
        .replace(' ', "")
        .replace('-', "")
        .replace('(', "")
        .replace(')', "");

    if let Some(stripped) = value.strip_prefix("+65") {
        value = stripped.to_string();
    } else if value.starts_with("65") && value.len() == 10 {
        value = value[2..].to_string();
    }

    // Singapore PayNow mobile numbers are stored as local 8-digit numbers.
    if value.len() != 8 || !value.chars().all(|character| character.is_ascii_digit()) {
        return Err("Enter an 8-digit Singapore mobile number.".to_string());
    }

    if !matches!(value.chars().next(), Some('8') | Some('9')) {
        return Err("PayNow mobile number should start with 8 or 9.".to_string());
    }

    Ok(value)
}

fn normalise_nric(input: &str) -> Result<String, String> {
    let value = input.trim().replace(' ', "").replace('-', "").to_uppercase();
    let characters: Vec<char> = value.chars().collect();

    if characters.len() != 9 {
        return Err("Enter a valid NRIC/FIN for PayNow.".to_string());
    }

    if !matches!(characters.first(), Some('S') | Some('T') | Some('F') | Some('G') | Some('M')) {
        return Err("Enter a valid NRIC/FIN for PayNow.".to_string());
    }

    if !characters[1..8].iter().all(|character| character.is_ascii_digit()) {
        return Err("Enter a valid NRIC/FIN for PayNow.".to_string());
    }

    if !characters[8].is_ascii_alphabetic() {
        return Err("Enter a valid NRIC/FIN for PayNow.".to_string());
    }

    Ok(value)
}
