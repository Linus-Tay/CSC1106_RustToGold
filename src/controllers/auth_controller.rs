use crate::controllers::session_guard::{
    admin_session_user_id, clear_admin_session, clear_customer_session, customer_session_user_id,
    redirect, store_admin_session, store_customer_session,
};
use crate::forms::{
    LoginForm, SignupAccountForm, SignupContactForm, SignupDeclarationForm, SignupDraft,
    SignupEmploymentForm, SignupForm, SignupPersonalForm, SignupSecurityForm,
};
use crate::services;
use crate::views::{
    render, AdminLoginTemplate, LoginTemplate, SignupAccountTemplate, SignupContactTemplate,
    SignupEmploymentTemplate, SignupPersonalTemplate, SignupReviewTemplate, SignupSecurityTemplate,
};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
const SIGNUP_DRAFT_KEY: &str = "signup_draft";

pub async fn login_page(session: Session) -> Result<HttpResponse> {
    if customer_session_user_id(&session).is_some() {
        return Ok(redirect("/customer/dashboard"));
    }

    render(LoginTemplate {
        error: String::new(),
        has_error: false,
    })
}

pub async fn admin_login_page(session: Session) -> Result<HttpResponse> {
    if admin_session_user_id(&session).is_some() {
        return Ok(redirect("/admin/dashboard"));
    }

    render(AdminLoginTemplate {
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
        Ok(user) if user.is_customer() => {
            store_customer_session(&session, &user)?;
            Ok(redirect("/customer/dashboard"))
        }
        Ok(_) => render(LoginTemplate {
            error: "Use the admin login page for staff access.".to_string(),
            has_error: true,
        }),
        Err(error) => render(LoginTemplate {
            error,
            has_error: true,
        }),
    }
}

pub async fn admin_login(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<LoginForm>,
) -> Result<HttpResponse> {
    match services::authenticate_user(&data.db, form.into_inner()).await {
        Ok(user) if user.is_staff_or_admin() => {
            store_admin_session(&session, &user)?;
            Ok(redirect("/admin/dashboard"))
        }
        Ok(_) => render(AdminLoginTemplate {
            error: "This login is only for staff and admin users.".to_string(),
            has_error: true,
        }),
        Err(error) => render(AdminLoginTemplate {
            error,
            has_error: true,
        }),
    }
}

pub async fn signup_page(session: Session) -> Result<HttpResponse> {
    if customer_session_user_id(&session).is_some() {
        return Ok(redirect("/customer/dashboard"));
    }

    Ok(redirect("/signup/account"))
}

pub async fn signup(session: Session) -> Result<HttpResponse> {
    signup_page(session).await
}

pub async fn show_signup_account(session: Session) -> Result<HttpResponse> {
    if customer_session_user_id(&session).is_some() {
        return Ok(redirect("/customer/dashboard"));
    }

    let draft = read_signup_draft(&session)?;
    render_account_page(&draft, None)
}

pub async fn post_signup_account(
    session: Session,
    form: web::Form<SignupAccountForm>,
) -> Result<HttpResponse> {
    let form = form.into_inner();
    let selected_account_type = clean_text(form.selected_account_type);
    let account_purpose = clean_text(form.account_purpose);

    if !matches!(
        selected_account_type.as_str(),
        "everyday_savings" | "high_yield_savings"
    ) {
        let draft = read_signup_draft(&session)?;
        return render_account_page(&draft, Some("Please choose an account type."));
    }

    if account_purpose.is_empty() {
        let draft = read_signup_draft(&session)?;
        return render_account_page(&draft, Some("Please select the main purpose of this account."));
    }

    let mut draft = read_signup_draft(&session)?;
    draft.selected_account_type = Some(selected_account_type);
    draft.preferred_account_name = clean_optional_text(form.preferred_account_name);
    draft.account_purpose = Some(account_purpose);
    save_signup_draft(&session, &draft)?;

    Ok(redirect("/signup/personal"))
}

pub async fn show_signup_personal(session: Session) -> Result<HttpResponse> {
    if customer_session_user_id(&session).is_some() {
        return Ok(redirect("/customer/dashboard"));
    }

    let draft = read_signup_draft(&session)?;
    render_personal_page(&draft, None)
}

pub async fn post_signup_personal(
    session: Session,
    form: web::Form<SignupPersonalForm>,
) -> Result<HttpResponse> {
    let form = form.into_inner();
    let full_name = clean_text(form.full_name);
    let nric_fin = clean_text(form.nric_fin).to_uppercase();
    let date_of_birth = clean_text(form.date_of_birth);
    let nationality = clean_text(form.nationality);
    let residential_status = clean_text(form.residential_status);
    let residential_address = clean_text(form.residential_address);

    let mut draft = read_signup_draft(&session)?;

    if full_name.len() < 2 {
        return render_personal_page(&draft, Some("Enter your full name as shown on your ID."));
    }

    if nric_fin.len() < 5 {
        return render_personal_page(&draft, Some("Enter a valid NRIC or FIN."));
    }

    if date_of_birth.is_empty()
        || nationality.is_empty()
        || residential_status.is_empty()
        || residential_address.is_empty()
    {
        return render_personal_page(&draft, Some("Please complete all required personal details."));
    }

    if form.identity_confirmed.is_none() {
        return render_personal_page(&draft, Some("Please confirm that the identity details are accurate."));
    }

    draft.full_name = Some(full_name);
    draft.nric_fin = Some(nric_fin);
    draft.date_of_birth = Some(date_of_birth);
    draft.nationality = Some(nationality);
    draft.residential_status = Some(residential_status);
    draft.residential_address = Some(residential_address);
    save_signup_draft(&session, &draft)?;

    Ok(redirect("/signup/contact"))
}

pub async fn show_signup_contact(session: Session) -> Result<HttpResponse> {
    if customer_session_user_id(&session).is_some() {
        return Ok(redirect("/customer/dashboard"));
    }

    let draft = read_signup_draft(&session)?;
    render_contact_page(&draft, None)
}

pub async fn post_signup_contact(
    session: Session,
    form: web::Form<SignupContactForm>,
) -> Result<HttpResponse> {
    let form = form.into_inner();
    let email = clean_text(form.email).to_lowercase();
    let phone_number = clean_text(form.phone_number);

    let mut draft = read_signup_draft(&session)?;

    if !email.contains('@') || email.len() < 5 {
        return render_contact_page(&draft, Some("Enter a valid email address."));
    }

    if phone_number.len() < 8 {
        return render_contact_page(&draft, Some("Enter a valid mobile number."));
    }

    draft.email = Some(email);
    draft.phone_number = Some(phone_number);
    draft.mailing_address = clean_optional_text(form.mailing_address);
    save_signup_draft(&session, &draft)?;

    Ok(redirect("/signup/employment"))
}

pub async fn show_signup_employment(session: Session) -> Result<HttpResponse> {
    if customer_session_user_id(&session).is_some() {
        return Ok(redirect("/customer/dashboard"));
    }

    let draft = read_signup_draft(&session)?;
    render_employment_page(&draft, None)
}

pub async fn post_signup_employment(
    session: Session,
    form: web::Form<SignupEmploymentForm>,
) -> Result<HttpResponse> {
    let form = form.into_inner();
    let employment_status = clean_text(form.employment_status);

    let mut draft = read_signup_draft(&session)?;

    if employment_status.is_empty() {
        return render_employment_page(&draft, Some("Please select your employment status."));
    }

    draft.employment_status = Some(employment_status);
    draft.occupation = clean_optional_text(form.occupation);
    draft.employer_name = clean_optional_text(form.employer_name);
    draft.monthly_income_range = clean_optional_text(form.monthly_income_range);
    draft.source_initial_deposit = clean_optional_text(form.source_initial_deposit);
    save_signup_draft(&session, &draft)?;

    Ok(redirect("/signup/security"))
}

pub async fn show_signup_security(session: Session) -> Result<HttpResponse> {
    if customer_session_user_id(&session).is_some() {
        return Ok(redirect("/customer/dashboard"));
    }

    render_security_page(None)
}

pub async fn post_signup_security(
    session: Session,
    form: web::Form<SignupSecurityForm>,
) -> Result<HttpResponse> {
    let form = form.into_inner();

    if form.setup_after_approval_acknowledged.is_none() {
        return render_security_page(Some(
            "Please acknowledge that online banking access is created only after admin approval.",
        ));
    }

    let mut draft = read_signup_draft(&session)?;
    draft.security_acknowledged = true;
    save_signup_draft(&session, &draft)?;

    Ok(redirect("/signup/review"))
}

pub async fn show_signup_review(session: Session) -> Result<HttpResponse> {
    if customer_session_user_id(&session).is_some() {
        return Ok(redirect("/customer/dashboard"));
    }

    let draft = read_signup_draft(&session)?;
    render_review_page(&draft, None)
}

pub async fn post_signup_submit(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<SignupDeclarationForm>,
) -> Result<HttpResponse> {
    let declaration_form = form.into_inner();
    let draft = read_signup_draft(&session)?;

    if declaration_form.opening_for_self.is_none()
        || declaration_form.not_acting_for_others.is_none()
        || declaration_form.funds_legitimate.is_none()
        || declaration_form.terms_agreed.is_none()
        || declaration_form.accuracy_confirmed.is_none()
    {
        return render_review_page(&draft, Some("Please confirm all declarations before submitting."));
    }

    let signup_form = match build_signup_form(draft, declaration_form) {
        Ok(signup_form) => signup_form,
        Err(error) => {
            let draft = read_signup_draft(&session)?;
            return render_review_page(&draft, Some(&error));
        }
    };

    match services::submit_customer_application(&data.db, signup_form).await {
        Ok((customer, product)) => {
            session.remove(SIGNUP_DRAFT_KEY);
            render(crate::views::OnboardingResultTemplate {
                reference_no: product.id.to_string(),
                created_at: customer.created_at.format("%d %b %Y, %I:%M %p").to_string(),
            })
        }
        Err(error) => {
            let draft = read_signup_draft(&session)?;
            render_review_page(&draft, Some(&error))
        }
    }
}

pub async fn logout(session: Session) -> Result<HttpResponse> {
    clear_customer_session(&session);
    Ok(redirect("/"))
}

pub async fn admin_logout(session: Session) -> Result<HttpResponse> {
    clear_admin_session(&session);
    Ok(redirect("/admin/login"))
}

fn read_signup_draft(session: &Session) -> Result<SignupDraft> {
    Ok(session
        .get::<SignupDraft>(SIGNUP_DRAFT_KEY)?
        .unwrap_or_default())
}

fn save_signup_draft(session: &Session, draft: &SignupDraft) -> Result<()> {
    session.insert(SIGNUP_DRAFT_KEY, draft)?;
    Ok(())
}

fn render_account_page(draft: &SignupDraft, error: Option<&str>) -> Result<HttpResponse> {
    render(SignupAccountTemplate {
        error: error.unwrap_or_default().to_string(),
        has_error: error.is_some(),
        selected_account_type: draft
            .selected_account_type
            .clone()
            .unwrap_or_else(|| "everyday_savings".to_string()),
        preferred_account_name: draft.preferred_account_name.clone().unwrap_or_default(),
        account_purpose: draft.account_purpose.clone().unwrap_or_default(),
    })
}

fn render_personal_page(draft: &SignupDraft, error: Option<&str>) -> Result<HttpResponse> {
    render(SignupPersonalTemplate {
        error: error.unwrap_or_default().to_string(),
        has_error: error.is_some(),
        full_name: draft.full_name.clone().unwrap_or_default(),
        nric_fin: draft.nric_fin.clone().unwrap_or_default(),
        date_of_birth: draft.date_of_birth.clone().unwrap_or_default(),
        nationality: draft.nationality.clone().unwrap_or_default(),
        residential_status: draft.residential_status.clone().unwrap_or_default(),
        residential_address: draft.residential_address.clone().unwrap_or_default(),
    })
}

fn render_contact_page(draft: &SignupDraft, error: Option<&str>) -> Result<HttpResponse> {
    render(SignupContactTemplate {
        error: error.unwrap_or_default().to_string(),
        has_error: error.is_some(),
        email: draft.email.clone().unwrap_or_default(),
        phone_number: draft.phone_number.clone().unwrap_or_default(),
        mailing_address: draft.mailing_address.clone().unwrap_or_default(),
    })
}

fn render_employment_page(draft: &SignupDraft, error: Option<&str>) -> Result<HttpResponse> {
    render(SignupEmploymentTemplate {
        error: error.unwrap_or_default().to_string(),
        has_error: error.is_some(),
        employment_status: draft.employment_status.clone().unwrap_or_default(),
        occupation: draft.occupation.clone().unwrap_or_default(),
        employer_name: draft.employer_name.clone().unwrap_or_default(),
        monthly_income_range: draft.monthly_income_range.clone().unwrap_or_default(),
        source_initial_deposit: draft.source_initial_deposit.clone().unwrap_or_default(),
    })
}

fn render_security_page(error: Option<&str>) -> Result<HttpResponse> {
    render(SignupSecurityTemplate {
        error: error.unwrap_or_default().to_string(),
        has_error: error.is_some(),
    })
}

fn render_review_page(draft: &SignupDraft, error: Option<&str>) -> Result<HttpResponse> {
    render(SignupReviewTemplate {
        error: error.unwrap_or_default().to_string(),
        has_error: error.is_some(),
        selected_account_type: account_type_label(
            draft.selected_account_type.as_deref().unwrap_or("everyday_savings"),
        )
        .to_string(),
        preferred_account_name: draft.preferred_account_name.clone().unwrap_or_default(),
        account_purpose: draft.account_purpose.clone().unwrap_or_default(),
        full_name: draft.full_name.clone().unwrap_or_default(),
        nric_fin: draft.nric_fin.clone().unwrap_or_default(),
        date_of_birth: draft.date_of_birth.clone().unwrap_or_default(),
        nationality: draft.nationality.clone().unwrap_or_default(),
        residential_status: draft.residential_status.clone().unwrap_or_default(),
        residential_address: draft.residential_address.clone().unwrap_or_default(),
        email: draft.email.clone().unwrap_or_default(),
        phone_number: draft.phone_number.clone().unwrap_or_default(),
        mailing_address: draft.mailing_address.clone().unwrap_or_default(),
        employment_status: draft.employment_status.clone().unwrap_or_default(),
        occupation: draft.occupation.clone().unwrap_or_default(),
        employer_name: draft.employer_name.clone().unwrap_or_default(),
        monthly_income_range: draft.monthly_income_range.clone().unwrap_or_default(),
        source_initial_deposit: draft.source_initial_deposit.clone().unwrap_or_default(),
    })
}

fn build_signup_form(
    draft: SignupDraft,
    declarations: SignupDeclarationForm,
) -> std::result::Result<SignupForm, String> {
    if !draft.security_acknowledged {
        return Err("Please complete the security notice before submitting.".to_string());
    }

    Ok(SignupForm {
        selected_account_type: require_field(draft.selected_account_type, "account type")?,
        full_name: require_field(draft.full_name, "full name")?,
        nric_fin: require_field(draft.nric_fin, "NRIC or FIN")?,
        date_of_birth: require_field(draft.date_of_birth, "date of birth")?,
        nationality: require_field(draft.nationality, "nationality")?,
        residential_status: require_field(draft.residential_status, "residential status")?,
        residential_address: require_field(draft.residential_address, "residential address")?,
        email: require_field(draft.email, "email address")?,
        phone_number: require_field(draft.phone_number, "mobile number")?,
        mailing_address: draft.mailing_address,
        employment_status: require_field(draft.employment_status, "employment status")?,
        occupation: draft.occupation,
        employer_name: draft.employer_name,
        monthly_income_range: draft.monthly_income_range,
        opening_for_self: declarations.opening_for_self,
        not_acting_for_others: declarations.not_acting_for_others,
        funds_legitimate: declarations.funds_legitimate,
        terms_agreed: declarations.terms_agreed,
        accuracy_confirmed: declarations.accuracy_confirmed,
    })
}

fn require_field(value: Option<String>, label: &str) -> std::result::Result<String, String> {
    match value {
        Some(value) if !value.trim().is_empty() => Ok(value),
        _ => Err(format!("Please complete the {} step before submitting.", label)),
    }
}

fn clean_text(value: String) -> String {
    value.trim().to_string()
}

fn clean_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn account_type_label(value: &str) -> &'static str {
    match value {
        "high_yield_savings" => "RustToGold High Yield Savings Account",
        _ => "RustToGold Everyday Savings Account",
    }
}
