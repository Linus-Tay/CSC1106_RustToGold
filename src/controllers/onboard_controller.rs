use crate::controllers::session_guard::{redirect, session_user_id};
use crate::forms::auth_forms::AccountCreationForm;
use crate::forms::onboard_forms::{Step3Form, Step4Form};
use crate::forms::{OnboardingForm, Step1Form, Step2Form};
use crate::models::{Customer, customer};
use crate::services::{self};
use crate::views::renderer::render_html;
use crate::views::templates::{AccountCreationSuccessTemplate, AccountCreationTemplate, OnboardingContactTemplate, OnboardingEmploymentTemplate, OnboardingReviewTemplate};
use crate::views::{ErrorTemplate, NotFoundTemplate, OnboardingAccountTemplate, OnboardingPersonalTemplate, OnboardingResultTemplate, OnboardingTemplate, render};
use actix_session::Session;
use askama::DynTemplate;
use chrono::NaiveDate;
use crate::AppState;
use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AccountCreationQueryParams {
    link: String
}


pub async fn onboarding(path: web::Path<String>, session: Session) -> Result<HttpResponse> {

    let onboarding_path = path.into_inner();
    let form_data = session.get::<OnboardingForm>("onboarding_form_data")?.unwrap_or_default();
    
    match onboarding_path.to_lowercase().as_str() {
        _ => {
            let path_template = get_path_template(&onboarding_path, form_data, None);

            match path_template {
                Some(template) => {
                    return render_html(template.dyn_render().map_err(actix_web::error::ErrorInternalServerError)?);
                    //return render_form(session, template.dyn_render().map_err(actix_web::error::ErrorInternalServerError)?).await
                }
                None => {
                    render(NotFoundTemplate)
                }
            }
        }
    }
}


pub async fn account_creation_init(query: web::Query<AccountCreationQueryParams>, data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    println!("im still runinnglol");
    match services::validate_account_creation_link(&data, &query.link).await {
        Ok(true) => {
            session.insert("account_creation_link", query.link.clone());
            Ok(redirect("/account-creation"))
        },
        _ => {
            render(NotFoundTemplate)
        }
    }
}


pub async fn account_creation(data: web::Data<AppState>, session: Session) -> Result<HttpResponse> {
    let account_creation_link = match session.get::<String>("account_creation_link")? {
        Some(link) => link,
        None => {
            return render(NotFoundTemplate);
        }
    };

    match services::get_customer_by_account_creation_link(&data, &account_creation_link).await {
        Ok(customer) => {
            render(AccountCreationTemplate {
                email: customer.email
            })
        },
        Err(e) => {
            render(NotFoundTemplate)
        }
    }

    // match services::validate_account_creation_link(&data, &query.link).await {
    //     Ok(true) => {
    //         session.insert("account_creation_link", query.link.clone());
    //         Ok(redirect("/account-creation/"))
    //     },
    //     Ok(false) => {
    //         render(NotFoundTemplate)
    //     }
    //     Err(e) => {
    //         render(NotFoundTemplate)
    //     }
    // }
}

pub async fn account_creation_submit(data: web::Data<AppState>, form: web::Form<AccountCreationForm>, session: Session) -> Result<HttpResponse> {
    let mut form = form.into_inner();
    let link = match session.get::<String>("account_creation_link")? {
        Some(link) => link,
        None => {
            return render(ErrorTemplate);
        }
    };

    let customer = match services::get_customer_by_account_creation_link(&data, &link).await {
        Ok(customer) => customer,
        Err(e) => {
            return render(ErrorTemplate)
        }
    };

    match services::register_user(&data.db, &customer.id, &customer.email, form).await {
        Ok(user) => {

            services::invalidate_account_creation_link(&data, &link)
            .await
            .map_err(|e| println!("An error occured while invalidating account creation link: {}", e));

            render(AccountCreationSuccessTemplate {
                username: user.username,
                email: user.email
            })
        },
        Err(e) => {
            println!("Error creating user: {}", e);
            render(ErrorTemplate)
        }
    }
}


pub async fn submit(data: web::Data<AppState>,session: Session) -> Result<HttpResponse> {
    let mut form_data = session.get::<OnboardingForm>("onboarding_form_data")?.unwrap_or_default();
    let product_id = match form_data.step1.as_ref().and_then(|s| s.selected_account_type.clone()) {
        Some(id) => id,
        None => {
            return Ok(redirect("/onboarding/account")); 
        }
    };
    let result = services::customer_service::create_customer_with_product(&data.db, &form_data, "savings".to_string()).await;

    match result {
        Ok((customer, product)) => {
            session.remove("onboarding_form_data");

            render(OnboardingResultTemplate {
                reference_no: product.id.to_string(),
                created_at: customer.created_at.to_string()
            })
        },
        Err(e) => {
            render_form(form_data, "review".to_string(), Some("Something went wrong with our end, we are unable to process this application currently. Please try again later.")).await

        }
    }
}

pub async fn step1_post(session: Session, form: web::Form<Step1Form>) -> Result<HttpResponse> {
    let mut form_data = session.get::<OnboardingForm>("onboarding_form_data")?
    .unwrap_or_default();

    let mut form = form.into_inner();
    let selected_account_type = clean_text(form.selected_account_type.clone().unwrap());
    let account_purpose = clean_text(form.account_purpose.clone());

    if !matches!(
        selected_account_type.as_str(),
        "everyday_savings" | "high_yield_savings"
    ) {
        return render_form(form_data, "account".to_string() , Some("Please choose an account type.")).await;
    }

    if account_purpose.is_empty() {
        return render_form(form_data, "account".to_string(), Some("Please select the main purpose of this account.")).await;
    }

    form.form_completed = true;

    form_data.step1 = Some(form);
    session.insert("onboarding_form_data", &form_data)?;

    Ok(redirect("/onboarding/personal"))
}

pub async fn step2_post(session: Session, form: web::Form<Step2Form>) -> Result<HttpResponse> {
    let mut form_data = session.get::<OnboardingForm>("onboarding_form_data")?
    .unwrap_or_default();

    let mut form = form.into_inner();
    let full_name = clean_text(form.full_name.clone());
    let nric_fin = clean_text(form.nric.clone()).to_uppercase();
    let date_of_birth = clean_text(form.dob.clone());
    let nationality = clean_text(form.nationality.clone());
    let residential_status = clean_text(form.residential_status.clone());
    let residential_address = clean_text(form.residential_address.clone());

    if full_name.len() < 2 {
        return render_form(form_data, "account".to_string(), Some("Enter your full name as shown on your ID.")).await;
    }

    if nric_fin.len() < 5 {
        return render_form(form_data, "account".to_string(), Some("Enter a valid NRIC or FIN.")).await;
    }

    if date_of_birth.is_empty()
        || nationality.is_empty()
        || residential_status.is_empty()
        || residential_address.is_empty()
    {
        return render_form(form_data, "account".to_string(), Some("Please complete all required personal details.")).await;
    }

    if form.identity_confirmed.is_none() {
        return render_form(form_data, "account".to_string(), Some("Please confirm that the identity details are accurate.")).await;
    }

    form.form_completed = true;

    form_data.step2 = Some(form);
    session.insert("onboarding_form_data", &form_data)?;

    Ok(redirect("/onboarding/contact"))
}

pub async fn step3_post(session: Session, form: web::Form<Step3Form>) -> Result<HttpResponse> {
    let mut form_data = session.get::<OnboardingForm>("onboarding_form_data")?
    .unwrap_or_default();

    let mut form = form.into_inner();
    let email = clean_text(form.email.clone()).to_lowercase();
    let phone_number = clean_text(form.phone_number.clone());


    if !email.contains('@') || email.len() < 5 {
        return render_form(form_data, "contact".to_string(), Some("Enter a valid email address.")).await;
    }

    if phone_number.len() < 8 {
        return render_form(form_data, "contact".to_string(), Some("Enter a valid mobile number.")).await;
    }

    form.form_completed = true;

    form_data.step3 = Some(form);
    session.insert("onboarding_form_data", &form_data)?;

    Ok(redirect("/onboarding/employment"))
}


pub async fn step4_post(session: Session, form: web::Form<Step4Form>) -> Result<HttpResponse> {
    let mut form_data = session.get::<OnboardingForm>("onboarding_form_data")?
    .unwrap_or_default();

    let mut form = form.into_inner();
    let employment_status = clean_text(form.employment_status.clone());


    if employment_status.is_empty() {
        return render_form(form_data, "employment".to_string(), Some("Please select your employment status.")).await;
    }

    form.form_completed = true;

    form_data.step4 = Some(form);
    session.insert("onboarding_form_data", &form_data)?;

    Ok(redirect("/onboarding/review"))
}

async fn render_form(form_data: OnboardingForm, form_path: String, error: Option<&str>) -> Result<HttpResponse> {
    let path_template = get_path_template(&form_path, form_data, error);

    match path_template {
        Some(template) => {
            return render_html(template.dyn_render().map_err(actix_web::error::ErrorInternalServerError)?);
        }
        None => {
            render(NotFoundTemplate)
        }
    }

}

fn get_path_template(id: &str, form_data: OnboardingForm, error: Option<&str>) -> Option<Box<dyn DynTemplate>> {

    let step1_data = form_data.step1.unwrap_or_default();
    let step2_data = form_data.step2.unwrap_or_default();
    let step3_data = form_data.step3.unwrap_or_default();
    let step4_data = form_data.step4.unwrap_or_default();

    println!("{}", step4_data.source_initial_deposit.clone().unwrap_or_default());

    match id.to_lowercase().as_str() {
        "account" => Some(Box::new(OnboardingAccountTemplate {
                error: error.unwrap_or_default().to_string(),
                has_error: error.is_some(),
                selected_account_type: step1_data.selected_account_type.unwrap_or("everyday_savings".to_string()),
                preferred_account_name: String::new(),
                account_purpose: step1_data.account_purpose
            })),
        "personal" => Some(Box::new(OnboardingPersonalTemplate {
                 error: error.unwrap_or_default().to_string(),
                has_error: error.is_some(),
                full_name: step2_data.full_name,
                nric: step2_data.nric,
                gender: step2_data.gender,
                race: step2_data.race,
                dob: step2_data.dob,
                nationality: step2_data.nationality,
                residential_status: step2_data.residential_status,
                residential_address: step2_data.residential_address,
                step1_completed: step1_data.form_completed,
                identity_confirmed: step2_data.identity_confirmed.is_some()
            })),
        "contact" => Some(Box::new(OnboardingContactTemplate {
                 error: error.unwrap_or_default().to_string(),
                has_error: error.is_some(),
                email: step3_data.email,
                phone_number: step3_data.phone_number,
                mailing_address: step3_data.mailing_address.unwrap_or_default(),
                step1_completed: step1_data.form_completed,
                step2_completed: step2_data.form_completed
            })),
        "employment" => Some(Box::new(OnboardingEmploymentTemplate {
                error: error.unwrap_or_default().to_string(),
                has_error: error.is_some(),
                employment_status: step4_data.employment_status,
                occupation: step4_data.occupation.unwrap_or_default(),
                employer_name: step4_data.employer_name.unwrap_or_default(),
                monthly_income_range: step4_data.monthly_income_range.unwrap_or_default(),
                source_initial_deposit: step4_data.source_initial_deposit.unwrap_or_default(),
                step1_completed: step1_data.form_completed,
                step2_completed: step2_data.form_completed,
                step3_completed: step3_data.form_completed
            })),
        "review" => Some(Box::new(OnboardingReviewTemplate {
             error: error.unwrap_or_default().to_string(),
            has_error: error.is_some(),
            selected_account_type: account_type_label(
                &step1_data.selected_account_type.unwrap_or("everyday_savings".to_string()),
            )
            .to_string(),
            preferred_account_name: String::new(),
            account_purpose: String::new(),
            full_name: step2_data.full_name,
            nric_fin: step2_data.nric,
            date_of_birth: step2_data.dob,
            nationality: step2_data.nationality,
            residential_status: step2_data.residential_status,
            residential_address: step2_data.residential_address,
            email: step3_data.email,
            phone_number: step3_data.phone_number,
            mailing_address: step3_data.mailing_address.unwrap_or_default(),
            employment_status: step4_data.employment_status,
            occupation: step4_data.occupation.unwrap_or_default(),
            employer_name: step4_data.employer_name.unwrap_or_default(),
            monthly_income_range: step4_data.monthly_income_range.unwrap_or_default(),
            source_initial_deposit: step4_data.source_initial_deposit.unwrap_or_default(),
            step1_completed: step1_data.form_completed,
            step2_completed: step2_data.form_completed,
            step3_completed: step3_data.form_completed,
            step4_completed: step4_data.form_completed
        })),
        _ => None,
    }
}

fn account_type_label(value: &str) -> &'static str {
    match value {
        "high_yield_savings" => "RustToGold High Yield Savings Account",
        _ => "RustToGold Everyday Savings Account",
    }
}

fn clean_text(value: String) -> String {
    value.trim().to_string()
}
// pub async fn redirect_to_product_information(session: Session) -> Result<HttpResponse> {
//     session.remove("onboarding_step");
//     if let Some(product_id  ) = query.product_id.as_deref().filter(|v| !v.is_empty()) {
//         session.insert("onboarding_product_id", product_id)?;
//     }

//     if let Some(channel) = query.channel.as_deref().filter(|v| !v.is_empty()) {
//         session.insert("onboarding_channel", channel)?;
//     }

//     Ok(redirect("/onboarding/product-information"))
// }

// pub async fn display_product(session: Session) -> Result<HttpResponse> {
//     let product_id = session.get::<String>("onboarding_product_id").ok().flatten();
//     //let channel = session.get::<String>("onboarding_channel").ok().flatten().unwrap_or_else(|| "PERSONAL".to_string());

//     let product = product_id.as_ref().and_then(|id| get_product_details(id));
//     match product {
//         Some(product) => {
//             render(OnboardingTemplate {
//                 product_available: true,
//                 product_id: product_id.clone().unwrap(),
//                 product_type: "savings".to_string(),
//                 product_name: product.name,
//                 product_summary: product.summary,
//                 product_rate: product.rate,
//                 product_minimum: product.minimum,
//                 product_features: product.features.clone(),
//             })
//         }
//         None => {
//             render(OnboardingTemplate {
//                 product_available: false,
//                 product_id: String::new(),
//                 product_type: "savings".to_string(),
//                 product_name: String::new(),
//                 product_summary: String::new(),
//                 product_rate: String::new(),
//                 product_minimum: String::new(),
//                 product_features: Vec::new(),
//             })
//         }
//     }
// }