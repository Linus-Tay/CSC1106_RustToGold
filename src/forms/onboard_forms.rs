use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
pub struct OnboardingForm {
    pub step1: Option<Step1Form>,
    pub step2: Option<Step2Form>,
    pub step3: Option<Step3Form>,
    pub step4: Option<Step4Form>
}

#[derive(Serialize, Deserialize, Default)]
pub struct Step1Form {
    pub selected_account_type: Option<String>,
    pub account_purpose: String,
    pub form_completed: bool
}

#[derive(Serialize, Deserialize, Default)]
pub struct Step2Form {
    pub full_name: String,
    pub nric: String,
    pub dob: String,
    pub gender: String,
    pub nationality: String,
    pub residential_status: String,
    pub race: String,
    pub residential_address: String,
    pub form_completed: bool,
    pub identity_confirmed: Option<String>
}

#[derive(Serialize, Deserialize, Default)]
pub struct Step3Form {
    pub phone_number: String,
    pub email: String,
    pub mailing_address: Option<String>,
    pub form_completed: bool
}

#[derive(Serialize, Deserialize, Default)]
pub struct Step4Form {
    pub employment_status: String,
    pub occupation: Option<String>,
    pub employer_name: Option<String>,
    pub industry: Option<String>,
    pub monthly_income_range: Option<String>,
    pub source_initial_deposit: Option<String>,
    pub form_completed: bool
}

