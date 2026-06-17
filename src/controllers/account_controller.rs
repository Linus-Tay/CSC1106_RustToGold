use std::iter::Empty;

use crate::controllers::session_guard::{redirect, session_user_id};
use crate::services::{self, get_product_details, get_path_template};
use crate::views::renderer::render_html;
use crate::views::{ErrorTemplate, OnboardingFormTemplate, OnboardingTemplate, render};
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct OnboardingQuery {
    #[serde(rename = "productId")]
    pub product_id: Option<String>,
    #[serde(rename = "Channel")]
    pub channel: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct OnboardingFormData {
    pub step1: Option<Step1Data>
}

#[derive(Serialize, Deserialize)]
pub struct Step1Data {
    pub full_name: String,
}

pub async fn onboarding(path: web::Path<String>, query: web::Query<OnboardingQuery>, session: Session) -> Result<HttpResponse> {

    let onboarding_path = path.into_inner();

    println!("test: {}", onboarding_path);
    
    match onboarding_path.to_lowercase().as_str() {
        "init" => {
            return redirect_to_product_information(query, session).await
        },
        "product-information" => {
            return display_product(session).await
        },
        _ => {
            let (path_template, step_number) = get_path_template(&onboarding_path);

            match path_template {
                Some(template) => {
                    return render_form(session, template.dyn_render().map_err(actix_web::error::ErrorInternalServerError)?, step_number).await
                }
                None => {
                    Ok(redirect("/onboarding/init"))
                }
            }
        }
    }
} 

pub async fn step1_post(session: Session, form: web::Form<Step1Data>) -> Result<HttpResponse> {
    let mut form_data = session.get::<OnboardingFormData>("onboarding_form_data")?
    .unwrap_or_default();

    form_data.step1 = Some(form.into_inner());
    session.insert("onboarding_form_data", &form_data)?;
    session.insert("onboarding_step", 2);

    Ok(redirect("/onboarding/additional-details"))
}

pub async fn render_form(session: Session, form_template: String, step_number: i32) -> Result<HttpResponse> {

    let product_id = session.get::<String>("onboarding_product_id").ok().flatten();
    let onboarding_step = session.get::<i32>("onboarding_step").ok().flatten();

    let product = product_id.as_ref().and_then(|id| get_product_details(id));

    match (onboarding_step, product) {
        (None, Some(_)) => {
            session.insert("onboarding_step", 1);    

            render_html(form_template)
        }
        (Some(step), Some(_)) if step == step_number => {    
            render_html(form_template)
        }
        _ => {
            render(ErrorTemplate )
        },
    }
}

pub async fn redirect_to_product_information(query: web::Query<OnboardingQuery>, session: Session) -> Result<HttpResponse> {
    session.remove("onboarding_step");
    if let Some(product_id  ) = query.product_id.as_deref().filter(|v| !v.is_empty()) {
        session.insert("onboarding_product_id", product_id)?;
    }

    if let Some(channel) = query.channel.as_deref().filter(|v| !v.is_empty()) {
        session.insert("onboarding_channel", channel)?;
    }

    Ok(redirect("/onboarding/product-information"))
}

pub async fn display_product(session: Session) -> Result<HttpResponse> {
    let product_id = session.get::<String>("onboarding_product_id").ok().flatten();
    let channel = session.get::<String>("onboarding_channel").ok().flatten().unwrap_or_else(|| "PERSONAL".to_string());

    let product = product_id.as_ref().and_then(|id| get_product_details(id));
    match product {
        Some(product) => {
            render(OnboardingTemplate {
                product_available: true,
                product_id: product_id.clone().unwrap(),
                channel,
                product_name: product.name,
                product_summary: product.summary,
                product_rate: product.rate,
                product_minimum: product.minimum,
                product_features: product.features.clone(),
            })
        }
        None => {
            render(OnboardingTemplate {
                product_available: false,
                product_id: String::new(),
                channel,
                product_name: String::new(),
                product_summary: String::new(),
                product_rate: String::new(),
                product_minimum: String::new(),
                product_features: Vec::new(),
            })
        }
    }
}
