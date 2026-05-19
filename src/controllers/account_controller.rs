use crate::controllers::session_guard::{redirect, session_user_id};
use crate::forms::{SignupForm};
use crate::services;
use crate::views::{render, OnboardingTemplate, SignupTemplate};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct OnboardingQuery {
    #[serde(rename = "productId")]
    pub product_id: Option<String>,
    #[serde(rename = "Channel")]
    pub channel: Option<String>,
}

pub async fn create_page(session: Session) -> Result<HttpResponse> {
    if session_user_id(&session).is_some() {
        return Ok(redirect("/customer/dashboard"));
    }

    render(SignupTemplate {
        error: String::new(),
        has_error: false,
    })
}

pub async fn display_product(query: web::Query<OnboardingQuery>) -> Result<HttpResponse> {
    let product_id = query.product_id.as_deref().filter(|v| !v.is_empty());
    let channel = query.channel.as_deref().unwrap_or("PERSONAL").to_string();

    if product_id.is_none() {
        return render(OnboardingTemplate {
            product_available: false,
            product_id: String::new(),
            channel,
            product_name: String::new(),
            product_summary: String::new(),
            product_rate: String::new(),
            product_minimum: String::new(),
            product_features: Vec::new(),
            action_url: String::from("/signup"),
        });
    }

    let product_id = product_id.unwrap();

    let (product_name, product_summary, product_rate, product_minimum, product_features) =
        match product_id.to_uppercase().as_str() {
            "XS" => (
                "Everyday Savings",
                "A flexible savings account for everyday spending and simple digital banking.",
                "0.75%",
                "1",
                vec![
                    String::from("No monthly fees"),
                    String::from("Instant debit card issuance"),
                    String::from("Online banking and mobile access"),
                    String::from("Contactless payments enabled"),
                ],
            ),
            "SM" => (
                "Smart Saver",
                "Higher interest for regular savers with easy access and low account costs.",
                "1.20%",
                "1",
                vec![
                    String::from("Tiered interest on balances"),
                    String::from("No monthly fees"),
                    String::from("Free card and account maintenance"),
                    String::from("Easy transfers and payments"),
                ],
            ),
            "PL" => (
                "Personal Loan",
                "A straight-through loan product for personal expenses with clear repayment terms.",
                "5.88%",
                "0",
                vec![
                    String::from("Fast approval process"),
                    String::from("Flexible tenor options"),
                    String::from("Competitive interest rate"),
                    String::from("Digital application support"),
                ],
            ),
            _ => (
                "Everyday Savings",
                "A flexible savings account for everyday spending and simple digital banking.",
                "0.75%",
                "1",
                vec![
                    String::from("No monthly fees"),
                    String::from("Instant debit card issuance"),
                    String::from("Online banking and mobile access"),
                    String::from("Contactless payments enabled"),
                ],
            ),
        };

    render(OnboardingTemplate {
        product_available: true,
        product_id: product_id.to_string(),
        channel: channel.to_string(),
        product_name: product_name.to_string(),
        product_summary: product_summary.to_string(),
        product_rate: product_rate.to_string(),
        product_minimum: product_minimum.to_string(),
        product_features,
        action_url: format!("/signup?productId={}", product_id),
    })
}

pub async fn create(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<SignupForm>,
) -> Result<HttpResponse> {
    match services::register_customer(&data.db, form.into_inner()).await {
        Ok(user) => {
            session.insert("user_id", user.id)?;
            session.insert("role", user.role)?;
            Ok(redirect("/customer/dashboard"))
        }
        Err(error) => render(SignupTemplate {
            error,
            has_error: true,
        }),
    }
}
