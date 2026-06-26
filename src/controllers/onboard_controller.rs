use crate::controllers::session_guard::{redirect, session_user_id};
use crate::forms::{OnboardingForm, Step1Form};
use crate::models::Customer;
use crate::services::{self, get_product_details, get_path_template};
use crate::views::renderer::render_html;
use crate::views::templates::AccountCreationTemplate;
use crate::views::{ErrorTemplate, OnboardingFormTemplate, OnboardingResultTemplate, OnboardingTemplate, render};
use actix_session::Session;
use askama::DynTemplate;
use chrono::NaiveDate;
use crate::AppState;
use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct OnboardingQuery {
    #[serde(rename = "productId")]
    pub product_id: Option<String>,
    #[serde(rename = "Channel")]
    pub channel: Option<String>,
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

pub async fn submit(data: web::Data<AppState>,session: Session) -> Result<HttpResponse> {
    let mut form_data = session.get::<OnboardingForm>("onboarding_form_data")?.unwrap_or_default();
    let product_id = match session.get::<String>("onboarding_product_id")? {
        Some(id) => id,
        None => return render(OnboardingResultTemplate {
            result_message: String::from("Missing product_id")
        }),
    };

    let result = services::customer_service::create_customer(&data.db, form_data).await;

    match result {
        Ok(customer) => {
            println!("{}", product_id);
            let test = services::product_service::create_product(&data.db, customer.id, product_id, "savings".to_string()).await;
            match test {
                Ok(test) => println!("nice"),
                Err(e) => println!("An error occured: {}", e)
            }

            session.remove("onboarding_step");
            session.remove("onboarding_form_data");
            session.remove("onboarding_product_id");
            session.remove("onboarding_product_type");

            render(OnboardingResultTemplate {
                result_message: String::from("Your application has been submitted. It will take 3 - 5 working days to process your application")
            })
        },
        Err(error_msg) => render(OnboardingResultTemplate {
            result_message: error_msg
        })
    }

    // let template = AccountCreationTemplate {
    //     account_creation_link: String::from("http://localhost:3000/")
    // };

    // let result = services::email_service::send_template_email(&String::from("jiayong.kok@hotmail.com"), &String::from("Welcome to Rust To Gold Bank!"), &template).await;


    // match result {
    //     Ok(()) => println!("works"),
    //     Err(e) => println!("{}", e.to_string())
    // }
    // render(ErrorTemplate)
}

pub async fn step1_post(session: Session, form: web::Form<Step1Form>) -> Result<HttpResponse> {
    let mut form_data = session.get::<OnboardingForm>("onboarding_form_data")?
    .unwrap_or_default();

    form_data.step1 = Some(form.into_inner());
    session.insert("onboarding_form_data", &form_data)?;
    session.insert("onboarding_step", 1);

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
        (Some(step), Some(_)) if step_number <= step => {
            println!("Works: {} {}", step, step_number);  
            render_html(form_template)
        }
        _ => {
            println!("{}, {}", onboarding_step.unwrap_or(0), step_number);
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
    //let channel = session.get::<String>("onboarding_channel").ok().flatten().unwrap_or_else(|| "PERSONAL".to_string());

    let product = product_id.as_ref().and_then(|id| get_product_details(id));
    match product {
        Some(product) => {
            render(OnboardingTemplate {
                product_available: true,
                product_id: product_id.clone().unwrap(),
                product_type: "savings".to_string(),
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
                product_type: "savings".to_string(),
                product_name: String::new(),
                product_summary: String::new(),
                product_rate: String::new(),
                product_minimum: String::new(),
                product_features: Vec::new(),
            })
        }
    }
}
