use crate::forms::CardApplicationForm;
use crate::models::{Card, Product};
use crate::repositories::{card_repository, product_repository};
use sqlx::PgPool;
use uuid::Uuid;

pub struct CardDashboardData {
    pub cards: Vec<Card>,
    pub has_cards: bool,
    pub accounts: Vec<Product>,
    pub has_accounts: bool,
}

pub async fn load_card_dashboard(db: &PgPool, customer_id: Uuid) -> Result<CardDashboardData, String> {
    let cards = card_repository::list_cards_by_customer(db, customer_id)
        .await
        .map_err(|_| "Could not load your cards.".to_string())?;
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

pub async fn create_card(db: &PgPool, customer_id: Uuid, form: CardApplicationForm) -> Result<Card, String> {
    let linked_product_id = Uuid::parse_str(form.linked_account_id.trim())
        .map_err(|_| "Choose a valid account to link this card to.".to_string())?;

    let card_type = match form.card_type.trim() {
        "debit" => "debit",
        "student" => "student",
        _ => return Err("Choose a valid card type.".to_string()),
    };

    let account = product_repository::get_active_product_for_customer_by_id(db, customer_id, linked_product_id)
        .await
        .map_err(|_| "The selected account is not active or does not belong to you.".to_string())?;

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

    let suffix = account.account_number.chars().rev().take(4).collect::<String>().chars().rev().collect::<String>();
    let masked_number = format!("**** **** **** {}", suffix);

    card_repository::create_card(db, customer_id, linked_product_id, card_type, &display_name, &masked_number)
        .await
        .map_err(|_| "Could not create the card.".to_string())
}

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
