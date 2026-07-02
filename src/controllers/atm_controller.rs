// Controller layer: handles HTTP/session flow and delegates business rules to services.

use crate::controllers::session_guard::redirect;
use crate::forms::atm_forms::{CardInsertionForm, PinValidationForm, ATMDepositForm};
use crate::repositories::{card_repository, product_repository};
use crate::services;
use crate::views::renderer::render_html;
use crate::views::templates::{
    ATMDepositSuccessTemplate, ATMDepositTemplate, ATMMenuTemplate, ATMPageTemplate, ATMPinTemplate, ATMWithdrawalSuccessTemplate, ATMWithdrawalTemplate, AccountCreationSetupTemplate, AccountCreationSuccessTemplate, OnboardingAccountTemplate, OnboardingContactTemplate, OnboardingEmploymentTemplate, OnboardingPersonalTemplate, OnboardingReviewTemplate,
};
use crate::views::{render, ErrorTemplate, NotFoundTemplate, OnboardingResultTemplate};
use crate::AppState;
use actix_session::Session;
use actix_web::cookie::Cookie;
use actix_web::cookie::time::Duration;
use actix_web::{web, HttpResponse, Result};
use askama::DynTemplate;
use serde::Deserialize;

// Renders the index ATM page.
pub async fn atm_page(session: Session) -> Result<HttpResponse> {
    render_index(false, String::new())
}

// Handles card insertion flow.
pub async fn card_validation(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<CardInsertionForm>,
) -> Result<HttpResponse> {
    let form = form.into_inner();
    let card_number = form.card_number.replace(" ", "");
    match services::validate_card(&data.db, &card_number).await {
        Ok(Some(card)) => {
            let _ = session.insert("card_number", card.card_number);
            Ok(redirect("/pin"))
        },
        _ => {
            render_index(true, "Invalid card".to_string())

        }
    }
}

// Renders the ATM pin page.
pub async fn pin_page(session: Session) -> Result<HttpResponse> {
    if let Ok(Some(card_number)) = session.get::<String>("card_number") {
        render(ATMPinTemplate {
            has_error: false,
            error: String::new()
        })
    }
    else {
        Ok(redirect("/"))
    }

}

// Handles card insertion flow.
pub async fn pin_validation(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<PinValidationForm>,
) -> Result<HttpResponse> {
    if let Ok(Some(card_number)) = session.get::<String>("card_number") {
        let form = form.into_inner();
        let pin_number = form.pin; // Fixed double semicolon
        
        match services::authenticate_card(&data.db, &card_number, &pin_number).await {
            Ok(card) => {
                let _ = session.insert("card_id", card.id.to_string()); 
                session.remove("card_number");
                Ok(redirect("/menu"))
            },
            _ => {
                render(ATMPinTemplate {
                    has_error: true,
                    error: "Invalid pin".to_string()
                })
            }
        }
    } else {
        return Ok(redirect("/"))
    }
}

// Renders the ATM menu page.
pub async fn menu_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
     if let Ok(Some(card_id)) = session.get::<String>("card_id") {
        let card_id = match uuid::Uuid::parse_str(&card_id) {
            Ok(value) => value,
            Err(_) => return Ok(redirect("/")),
        };

        let card = match card_repository::find_active_by_card_id(&data.db, &card_id).await {
            Ok(Some(card)) => card, 
            _ => {
                return Ok(redirect("/eject")); 
            }
        };

        let product = match product_repository::get_product_by_account_number(&data.db, &card.account_number).await {
            Ok(Some(product)) => product, 
            _ => {
                return Ok(redirect("/eject"));
            }
        };

        render(ATMMenuTemplate {
            account_balance: String::from(product.balance_display()),
            card_number_last_4: card.card_number[card.card_number.len() - 4..].to_string()
        })
     }
     else {
        return Ok(redirect("/"))
     }
}

pub async fn eject(session: Session) -> Result<HttpResponse> {
    session.purge();
    Ok(redirect("/"))
}

// Renders the ATM pin page.
pub async fn atm_deposit_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    if let Ok(Some(card_id)) = session.get::<String>("card_id") {

        let card_id = match uuid::Uuid::parse_str(&card_id) {
            Ok(value) => value,
            Err(_) => return Ok(redirect("/")),
        };

        let card = match card_repository::find_active_by_card_id(&data.db, &card_id).await {
            Ok(Some(card)) => card, 
            _ => {
                return Ok(redirect("/eject")); 
            }
        };

        render(ATMDepositTemplate {
            has_error: false,
            error: String::new(),
            card_number_last_4: card.card_number[card.card_number.len() - 4..].to_string()
        })
    }
    else {
        Ok(redirect("/"))
    }
}

pub async fn atm_deposit(data: web::Data<AppState>, form: web::Form<ATMDepositForm>, session: Session) -> Result<HttpResponse> {
    let form = form.into_inner();
    let amount = form.amount;
    if let Ok(Some(card_id)) = session.get::<String>("card_id") {
        let card_id = match uuid::Uuid::parse_str(&card_id) {
            Ok(value) => value,
            Err(_) => return Ok(redirect("/")),
        };

        let card = match card_repository::find_active_by_card_id(&data.db, &card_id).await {
            Ok(Some(card)) => card, 
            _ => {
                return Ok(redirect("/eject")); 
            }
        };

        match services::atm_deposit(&data, card.customer_id, &amount, &card.account_number).await {
            Ok(account) => {
                render(ATMDepositSuccessTemplate {
                    account_balance: account.balance_display(),
                    card_number_last_4: card.card_number[card.card_number.len() - 4..].to_string(),
                    amount: amount
                })
            }
            Err(error) => {
                render(ATMDepositTemplate {
                    has_error: false,
                    error: error,
                    card_number_last_4: card.card_number[card.card_number.len() - 4..].to_string()
                })
            }
        }
    }
    else {
        Ok(redirect("/"))
    }
}

// Renders the ATM pin page.
pub async fn atm_withdrawal_page(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    if let Ok(Some(card_id)) = session.get::<String>("card_id") {

        let card_id = match uuid::Uuid::parse_str(&card_id) {
            Ok(value) => value,
            Err(_) => return Ok(redirect("/")),
        };

        let card = match card_repository::find_active_by_card_id(&data.db, &card_id).await {
            Ok(Some(card)) => card, 
            _ => {
                return Ok(redirect("/eject")); 
            }
        };

        
        let product = match product_repository::get_product_by_account_number(&data.db, &card.account_number).await {
            Ok(Some(product)) => product, 
            _ => {
                return Ok(redirect("/eject"));
            }
        };

        render(ATMWithdrawalTemplate {
            has_error: false,
            error: String::new(),
            account_balance: product.balance_display(),
            card_number_last_4: card.card_number[card.card_number.len() - 4..].to_string()
        })
    }
    else {
        Ok(redirect("/"))
    }
}

pub async fn atm_withdraw(data: web::Data<AppState>, form: web::Form<ATMDepositForm>, session: Session) -> Result<HttpResponse> {
    let form = form.into_inner();
    let amount = form.amount;
    if let Ok(Some(card_id)) = session.get::<String>("card_id") {
        let card_id = match uuid::Uuid::parse_str(&card_id) {
            Ok(value) => value,
            Err(_) => return Ok(redirect("/")),
        };

        let card = match card_repository::find_active_by_card_id(&data.db, &card_id).await {
            Ok(Some(card)) => card, 
            _ => {
                return Ok(redirect("/eject")); 
            }
        };

        match services::atm_withdraw(&data, card.customer_id, &amount, &card.account_number).await {
            Ok(account) => {
                render(ATMWithdrawalSuccessTemplate {
                    account_balance: account.balance_display(),
                    card_number_last_4: card.card_number[card.card_number.len() - 4..].to_string(),
                    amount: amount
                })
            }
            Err(error) => {
                let product = match product_repository::get_product_by_account_number(&data.db, &card.account_number).await {
                    Ok(Some(product)) => product, 
                    _ => {
                        return Ok(redirect("/eject"));
                    }
                };

                render(ATMWithdrawalTemplate {
                    has_error: false,
                    account_balance: product.balance_display(),
                    error: error,
                    card_number_last_4: card.card_number[card.card_number.len() - 4..].to_string()
                })
            }
        }
    }
    else {
        Ok(redirect("/"))
    }
}



fn render_index(has_error: bool, error_message: String) -> Result<HttpResponse> {
    return render(ATMPageTemplate {
        has_error: has_error,
        error: error_message.to_string()
    })
}