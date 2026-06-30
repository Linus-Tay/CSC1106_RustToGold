use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AccountCreationForm {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
    pub notify_transactions: Option<String>,
    pub notify_login: Option<String>,
    pub notify_promotions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignupDraft {
    pub selected_account_type: Option<String>,
    pub preferred_account_name: Option<String>,
    pub account_purpose: Option<String>,

    pub full_name: Option<String>,
    pub nric_fin: Option<String>,
    pub date_of_birth: Option<String>,
    pub nationality: Option<String>,
    pub residential_status: Option<String>,
    pub residential_address: Option<String>,

    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub mailing_address: Option<String>,

    pub employment_status: Option<String>,
    pub occupation: Option<String>,
    pub employer_name: Option<String>,
    pub monthly_income_range: Option<String>,
    pub source_initial_deposit: Option<String>,

    pub security_acknowledged: bool,
}

#[derive(Debug, Deserialize)]
pub struct SignupAccountForm {
    pub selected_account_type: String,
    pub preferred_account_name: Option<String>,
    pub account_purpose: String,
}

#[derive(Debug, Deserialize)]
pub struct SignupPersonalForm {
    pub full_name: String,
    pub nric_fin: String,
    pub date_of_birth: String,
    pub nationality: String,
    pub residential_status: String,
    pub residential_address: String,
    pub identity_confirmed: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignupContactForm {
    pub email: String,
    pub phone_number: String,
    pub mailing_address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignupEmploymentForm {
    pub employment_status: String,
    pub occupation: Option<String>,
    pub employer_name: Option<String>,
    pub monthly_income_range: Option<String>,
    pub source_initial_deposit: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignupSecurityForm {
    pub setup_after_approval_acknowledged: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignupDeclarationForm {
    pub opening_for_self: Option<String>,
    pub not_acting_for_others: Option<String>,
    pub funds_legitimate: Option<String>,
    pub terms_agreed: Option<String>,
    pub accuracy_confirmed: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignupForm {
    pub selected_account_type: String,
    pub full_name: String,
    pub nric_fin: String,
    pub date_of_birth: String,
    pub nationality: String,
    pub residential_status: String,
    pub residential_address: String,
    pub email: String,
    pub phone_number: String,
    pub mailing_address: Option<String>,
    pub employment_status: String,
    pub occupation: Option<String>,
    pub employer_name: Option<String>,
    pub monthly_income_range: Option<String>,
    pub opening_for_self: Option<String>,
    pub not_acting_for_others: Option<String>,
    pub funds_legitimate: Option<String>,
    pub terms_agreed: Option<String>,
    pub accuracy_confirmed: Option<String>,
}
