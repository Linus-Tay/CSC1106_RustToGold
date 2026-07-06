
use crate::forms::GiroArrangementForm;
use crate::models::{GiroArrangement, Money, Product};
use crate::repositories::{giro_repository, product_repository};
use crate::services::{support::clean_optional_text, transaction_control_service};
use chrono::{Duration, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct GiroDashboard {
    pub accounts: Vec<Product>,
    pub arrangements: Vec<GiroArrangement>,
}

// Load load giro dashboard
pub async fn load_giro_dashboard(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<GiroDashboard, String> {
    let accounts = product_repository::list_active_products_by_customer(db, &customer_id)
        .await
        .map_err(|_| "Could not load your active bank accounts.".to_string())?;
    let arrangements = giro_repository::list_by_customer(db, customer_id)
        .await
        .map_err(|_| "Could not load your GIRO arrangements.".to_string())?;

    Ok(GiroDashboard { accounts, arrangements })
}

// Handle create giro arrangement
pub async fn create_giro_arrangement(
    db: &PgPool,
    customer_id: Uuid,
    form: GiroArrangementForm,
) -> Result<(), String> {
    let from_product_id = Uuid::parse_str(form.from_product_id.trim())
        .map_err(|_| "Choose a valid source account.".to_string())?;
    let payee_name = form.payee_name.trim();
    // GIRO should name the billing organisation clearly
    if payee_name.len() < 2 {
        return Err("Enter the billing organisation or payee name.".to_string());
    }

    let source = product_repository::get_active_product_for_customer_by_id(db, customer_id, from_product_id)
        .await
        .map_err(|_| "Choose an active account that belongs to you.".to_string())?;

    let recipient_account_number = form.recipient_account_number.trim();
    let recipient = product_repository::get_product_by_account_number(db, recipient_account_number)
        .await
        .map_err(|_| "Could not check the recipient account.".to_string())?
        .ok_or_else(|| "No RustToGold account was found for this recipient number.".to_string())?;

    // Keep GIRO as an external recurring payment, not self-transfer
    if recipient.customer_id == customer_id {
        return Err("GIRO arrangements cannot be created to your own account.".to_string());
    }
    if recipient.status != "active" {
        return Err("Recipient account must be active before a GIRO arrangement can be created.".to_string());
    }

    let amount = Money::parse_dollars(&form.amount)?;
    // Simple bank-like range for a standing instruction
    if amount.cents() < 1_00 {
        return Err("GIRO amount must be at least $1.00.".to_string());
    }
    if amount.cents() > 10_000_00 {
        return Err("Single GIRO payment amount is capped at $10,000.00.".to_string());
    }

    let frequency = normalise_frequency(&form.frequency)?;
    let start_date = parse_start_date(&form.start_date)?;
    let end_date = parse_end_date(&form.end_date, start_date)?;
    let note = clean_optional_text(&form.note);

    // Reuse the same Money Lock, daily limit and fraud rules before setup
    transaction_control_service::validate_outgoing_transaction(
        db,
        customer_id,
        Some(source.id),
        amount.cents(),
        note.as_deref(),
        "GIRO setup",
        true,
    )
    .await?;

    giro_repository::insert_arrangement(
        db,
        customer_id,
        source.id,
        recipient.id,
        payee_name,
        amount.cents(),
        &frequency,
        start_date,
        end_date,
        note.as_deref(),
    )
    .await
    .map_err(|error| {
        eprintln!("GIRO setup failed: {error:?}");
        "Could not create the GIRO arrangement.".to_string()
    })?;

    Ok(())
}

// Handle cancel giro arrangement
pub async fn cancel_giro_arrangement(
    db: &PgPool,
    customer_id: Uuid,
    arrangement_id: Uuid,
) -> Result<(), String> {
    let changed = giro_repository::cancel_arrangement(db, customer_id, arrangement_id)
        .await
        .map_err(|_| "Could not cancel this GIRO arrangement.".to_string())?;

    if changed {
        Ok(())
    } else {
        Err("No active GIRO arrangement was found to cancel.".to_string())
    }
}

// Process normalise frequency
fn normalise_frequency(input: &str) -> Result<String, String> {
    match input.trim() {
        "weekly" => Ok("weekly".to_string()),
        "monthly" => Ok("monthly".to_string()),
        _ => Err("Choose weekly or monthly GIRO frequency.".to_string()),
    }
}

// Validate parse start date
fn parse_start_date(input: &str) -> Result<NaiveDate, String> {
    let date = NaiveDate::parse_from_str(input.trim(), "%Y-%m-%d")
        .map_err(|_| "Choose a valid GIRO start date.".to_string())?;
    let today = Utc::now().date_naive();
    // Standing instructions should start today or later
    if date < today {
        return Err("GIRO start date cannot be in the past.".to_string());
    }
    if date > today + Duration::days(365) {
        return Err("GIRO start date must be within the next 12 months.".to_string());
    }
    Ok(date)
}

// Validate parse end date
fn parse_end_date(input: &str, start_date: NaiveDate) -> Result<Option<NaiveDate>, String> {
    let value = input.trim();
    if value.is_empty() {
        return Ok(None);
    }

    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| "Choose a valid GIRO end date.".to_string())?;
    if date < start_date {
        return Err("GIRO end date cannot be before the start date.".to_string());
    }
    if date > start_date + Duration::days(1095) {
        return Err("GIRO end date must be within 3 years of the start date.".to_string());
    }
    Ok(Some(date))
}
