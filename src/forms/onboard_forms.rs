use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
pub struct OnboardingForm {
    pub step1: Option<Step1Form>,
    pub step2: Option<Step2Form>
}

#[derive(Serialize, Deserialize)]
pub struct Step1Form {
    pub full_name: String,
    pub email: String,
    pub nric: String,
    pub dob: String,
    pub nationality: String,
    pub residential_status: String,
    pub race: String
}

#[derive(Serialize, Deserialize)]
pub struct Step2Form {
    pub full_name: String,
    pub nric: String
}