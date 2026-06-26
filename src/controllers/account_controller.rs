use crate::controllers::session_guard::{redirect, session_user_id};
use crate::forms::SignupForm;
use crate::models::find_product;
use crate::services;
use crate::views::{render, SignupTemplate};
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

    render(empty_signup_template(String::new(), false))
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
        Err(error) => render(empty_signup_template(error, true)),
    }
}

fn empty_signup_template(error: String, has_error: bool) -> SignupTemplate {
    SignupTemplate {
        error,
        has_error,
        has_selected_product: false,
        selected_product_id: String::new(),
        selected_product_name: String::new(),
        selected_product_summary: String::new(),
    }
}
