use crate::controllers::session_guard::{redirect, session_user_id};
use crate::forms::{LoginForm, SignupForm};
use crate::models::find_product;
use crate::services;
use crate::views::{render, LoginTemplate, SignupTemplate};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SignupQuery {
    #[serde(rename = "productId")]
    pub product_id: Option<String>,
}

pub async fn login_page(session: Session) -> Result<HttpResponse> {
    if session_user_id(&session).is_some() {
        return Ok(redirect("/customer/dashboard"));
    }

    render(LoginTemplate {
        error: String::new(),
        has_error: false,
    })
}

pub async fn login(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<LoginForm>,
) -> Result<HttpResponse> {
    match services::authenticate_user(&data.db, form.into_inner()).await {
        Ok(user) => {
            session.insert("user_id", user.id)?;
            session.insert("role", user.role)?;
            Ok(redirect("/customer/dashboard"))
        }
        Err(error) => render(LoginTemplate {
            error,
            has_error: true,
        }),
    }
}

pub async fn signup_page(session: Session, query: web::Query<SignupQuery>) -> Result<HttpResponse> {
    if session_user_id(&session).is_some() {
        return Ok(redirect("/customer/dashboard"));
    }

    render(signup_template(String::new(), false, query.product_id.as_deref()))
}

pub async fn signup(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<SignupForm>,
) -> Result<HttpResponse> {
    let form_data = form.into_inner();
    let selected_product_id = form_data.product_id.clone();

    match services::register_customer(&data.db, form_data).await {
        Ok(user) => {
            session.insert("user_id", user.id)?;
            session.insert("role", user.role)?;
            Ok(redirect("/customer/dashboard"))
        }
        Err(error) => render(signup_template(error, true, selected_product_id.as_deref())),
    }
}

pub async fn logout(session: Session) -> Result<HttpResponse> {
    session.purge();
    Ok(redirect("/"))
}

fn signup_template(error: String, has_error: bool, product_id: Option<&str>) -> SignupTemplate {
    if let Some(product_id) = product_id {
        if let Some(product) = find_product(product_id) {
            return SignupTemplate {
                error,
                has_error,
                has_selected_product: true,
                selected_product_id: product.id.to_string(),
                selected_product_name: product.name.to_string(),
                selected_product_summary: product.summary.to_string(),
            };
        }
    }

    SignupTemplate {
        error,
        has_error,
        has_selected_product: false,
        selected_product_id: String::new(),
        selected_product_name: String::new(),
        selected_product_summary: String::new(),
    }
}
