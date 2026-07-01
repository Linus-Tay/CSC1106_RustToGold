use crate::forms::{MoneyLockForm, TransactionLimitForm};
use crate::models::{FraudAlert, Money, TransactionControl};
use chrono::Utc;
use crate::repositories::transaction_control_repository;
use sqlx::PgPool;
use uuid::Uuid;

const MIN_DAILY_LIMIT_CENTS: i64 = 100_00;
const MAX_DAILY_LIMIT_CENTS: i64 = 50_000_00;
const HIGH_VALUE_NOTE_CENTS: i64 = 10_000_00;
const AML_HOLD_CENTS: i64 = 25_000_00;
const RAPID_TRANSFER_COUNT_LIMIT: i64 = 5;
const RAPID_TRANSFER_WINDOW_MINUTES: i64 = 10;
const RAPID_TRANSFER_AMOUNT_CENTS: i64 = 20_000_00;

pub struct TransactionControlsPageData {
    pub controls: TransactionControl,
    pub outgoing_today_cents: i64,
    pub outgoing_today_display: String,
    pub remaining_today_display: String,
    pub alerts: Vec<FraudAlert>,
}

pub async fn load_transaction_controls_page(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<TransactionControlsPageData, String> {
    let controls = load_effective_controls(db, customer_id).await?;
    let outgoing_today_cents = transaction_control_repository::sum_outgoing_today(db, customer_id)
        .await
        .map_err(|_| "Could not load today's outgoing transfer total.".to_string())?;
    let alerts = transaction_control_repository::list_recent_alerts(db, customer_id, 8)
        .await
        .map_err(|_| "Could not load fraud monitoring records.".to_string())?;
    let remaining = controls.daily_limit_cents.saturating_sub(outgoing_today_cents);

    Ok(TransactionControlsPageData {
        controls,
        outgoing_today_cents,
        outgoing_today_display: Money::from_cents(outgoing_today_cents).display(),
        remaining_today_display: Money::from_cents(remaining).display(),
        alerts,
    })
}

pub async fn update_daily_transaction_limit(
    db: &PgPool,
    customer_id: Uuid,
    form: TransactionLimitForm,
) -> Result<TransactionControl, String> {
    let requested = Money::parse_dollars(&form.daily_limit)?;
    let requested_cents = requested.cents();

    // Keep customer-defined limits inside a safe range for this banking flow.
    if !(MIN_DAILY_LIMIT_CENTS..=MAX_DAILY_LIMIT_CENTS).contains(&requested_cents) {
        return Err("Daily transaction limit must be between $100.00 and $50,000.00.".to_string());
    }

    let controls = load_effective_controls(db, customer_id).await?;

    // The new limit takes effect now, but the customer cannot keep changing it repeatedly.
    if controls
        .limit_change_effective_at
        .map(|value| value > Utc::now())
        .unwrap_or(false)
    {
        return Err(format!(
            "Your daily limit was updated recently. You can change it again after {}.",
            controls.limit_change_effective_display()
        ));
    }

    transaction_control_repository::set_daily_limit_immediate(db, customer_id, requested_cents)
        .await
        .map_err(|_| "Could not update the daily transaction limit.".to_string())
}

pub async fn update_money_lock(
    db: &PgPool,
    customer_id: Uuid,
    form: MoneyLockForm,
) -> Result<TransactionControl, String> {
    let controls = load_effective_controls(db, customer_id).await?;

    match form.action.as_str() {
        // Locking should be instant because the customer is trying to protect funds.
        "enable" => transaction_control_repository::enable_money_lock(db, customer_id)
            .await
            .map_err(|_| "Could not enable Money Lock.".to_string()),
        "request_unlock" | "disable" => {
            // The account owner can unlock immediately after confirming the action.
            if !controls.money_lock_enabled {
                return Ok(controls);
            }
            transaction_control_repository::request_money_unlock(db, customer_id)
                .await
                .map_err(|_| "Could not unlock Money Lock.".to_string())
        }
        _ => Err("Choose a valid Money Lock action.".to_string()),
    }
}

pub async fn validate_outgoing_transaction(
    db: &PgPool,
    customer_id: Uuid,
    product_id: Option<Uuid>,
    amount_cents: i64,
    note: Option<&str>,
    channel: &str,
    counts_towards_limit: bool,
) -> Result<(), String> {
    let controls = load_effective_controls(db, customer_id).await?;

    if amount_cents <= 0 {
        return Err("Transfer amount must be greater than $0.00.".to_string());
    }

    // Money Lock is a customer-controlled block, so we return a clear error without sending it to AML monitoring.
    if controls.money_lock_enabled {
        let message = "Money Lock is enabled. Outgoing transfers, PayNow transfers and GIRO setup are blocked until you unlock it.";
        return Err(message.to_string());
    }

    if !counts_towards_limit {
        return Ok(());
    }

    let outgoing_today = transaction_control_repository::sum_outgoing_today(db, customer_id)
        .await
        .map_err(|_| "Could not verify today's transfer usage.".to_string())?;

    // Daily limit is normal customer validation, not an AML admin alert.
    if outgoing_today.saturating_add(amount_cents) > controls.daily_limit_cents {
        let message = format!(
            "This transfer exceeds your active daily limit of {}. Reduce the amount or update your transaction limit first.",
            controls.daily_limit_display()
        );
        return Err(message);
    }

    // High-value transfers need a reference before they can be logged for review.
    let note_missing = note.map(|value| value.trim().is_empty()).unwrap_or(true);
    if amount_cents >= HIGH_VALUE_NOTE_CENTS && note_missing {
        let message = "For transfers of $10,000.00 or more, add a clear payment reference before submitting.";
        return Err(message.to_string());
    }

    // Very large external transfers are held for manual review before money moves.
    if amount_cents >= AML_HOLD_CENTS {
        let message = "This high-value transaction has been held for bank review. Please contact support or try a lower amount.";
        record_alert(db, customer_id, product_id, "HIGH_VALUE_REVIEW", "high", channel, amount_cents, message).await;
        return Err(message.to_string());
    }

    let recent_count = transaction_control_repository::count_outgoing_since_minutes(
        db,
        customer_id,
        RAPID_TRANSFER_WINDOW_MINUTES,
    )
    .await
    .map_err(|_| "Could not verify recent transaction velocity.".to_string())?;

    // Velocity rules catch repeated quick transfers that look abnormal.
    if recent_count >= RAPID_TRANSFER_COUNT_LIMIT {
        let message = "Too many outgoing transfers were attempted within 10 minutes. Please wait before trying again.";
        record_alert(db, customer_id, product_id, "VELOCITY_COUNT", "medium", channel, amount_cents, message).await;
        return Err(message.to_string());
    }

    let recent_total = transaction_control_repository::sum_outgoing_since_minutes(
        db,
        customer_id,
        RAPID_TRANSFER_WINDOW_MINUTES,
    )
    .await
    .map_err(|_| "Could not verify recent transfer amount.".to_string())?;

    // Also block rapid transfers when the total amount spikes.
    if recent_total.saturating_add(amount_cents) > RAPID_TRANSFER_AMOUNT_CENTS {
        let message = "Recent outgoing transfer amount is unusually high. Please wait before trying another transfer.";
        record_alert(db, customer_id, product_id, "VELOCITY_AMOUNT", "high", channel, amount_cents, message).await;
        return Err(message.to_string());
    }

    // Allowed high-value external transfers are flagged for admin follow-up.
    if amount_cents >= HIGH_VALUE_NOTE_CENTS {
        let message = "High-value transaction completed and flagged for standard monitoring.";
        record_alert_with_status(
            db,
            customer_id,
            product_id,
            "HIGH_VALUE_MONITORING",
            "low",
            channel,
            amount_cents,
            message,
            "flagged",
        )
        .await;
    }

    Ok(())
}

async fn load_effective_controls(db: &PgPool, customer_id: Uuid) -> Result<TransactionControl, String> {
    transaction_control_repository::get_or_create_controls(db, customer_id)
        .await
        .map_err(|_| "Could not initialise transaction controls.".to_string())?;
    transaction_control_repository::apply_ready_cooldowns(db, customer_id)
        .await
        .map_err(|_| "Could not refresh transaction control cooldowns.".to_string())?;
    transaction_control_repository::get_or_create_controls(db, customer_id)
        .await
        .map_err(|_| "Could not load transaction controls.".to_string())
}

async fn record_alert(
    db: &PgPool,
    customer_id: Uuid,
    product_id: Option<Uuid>,
    rule_code: &str,
    severity: &str,
    channel: &str,
    amount_cents: i64,
    message: &str,
) {
    let _ = transaction_control_repository::insert_fraud_alert(
        db,
        customer_id,
        product_id,
        rule_code,
        severity,
        channel,
        amount_cents,
        message,
    )
    .await;
}

async fn record_alert_with_status(
    db: &PgPool,
    customer_id: Uuid,
    product_id: Option<Uuid>,
    rule_code: &str,
    severity: &str,
    channel: &str,
    amount_cents: i64,
    message: &str,
    status: &str,
) {
    let _ = transaction_control_repository::insert_fraud_alert_with_status(
        db,
        customer_id,
        product_id,
        rule_code,
        severity,
        channel,
        amount_cents,
        message,
        status,
    )
    .await;
}
