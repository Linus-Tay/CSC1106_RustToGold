use askama::DynTemplate;

use crate::{forms::OnboardingForm, views::{ErrorTemplate, OnboardingAccountTemplate, OnboardingPersonalTemplate, templates::{OnboardingContactTemplate, OnboardingEmploymentTemplate, OnboardingReviewTemplate}}};

pub fn get_path_template(id: &str, form_data: OnboardingForm, error: Option<&str>) -> Option<Box<dyn DynTemplate>> {

    let step1_data = form_data.step1.unwrap_or_default();
    let step2_data = form_data.step2.unwrap_or_default();
    let step3_data = form_data.step3.unwrap_or_default();
    let step4_data = form_data.step4.unwrap_or_default();

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
                source_initial_deposit: step4_data.source_of_funds.unwrap_or_default(),
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
            source_initial_deposit: step4_data.source_of_funds.unwrap_or_default(),
            step1_completed: step1_data.form_completed,
            step2_completed: step2_data.form_completed,
            step3_completed: step3_data.form_completed,
            step4_completed: step4_data.form_completed
        })),
        _ => None,
    }
}

// pub fn get_product_details(id: &str) -> Option<ProductDetails> {
//     match id.to_uppercase().as_str() {
//         "XS" => Some(ProductDetails {
//             name: "Everyday Savings".to_string(),
//             product_type: "savings".to_string(),
//             summary: "A flexible savings account for everyday spending and simple digital banking.".to_string(),
//             rate: "0.75%".to_string(),
//             minimum: "1".to_string(),
//             features: vec![
//                 "No monthly fees".to_string(),
//                 "Instant debit card issuance".to_string(),
//                 "Online banking and mobile access".to_string(),
//                 "Contactless payments enabled".to_string(),
//             ],
//         }),
//         "SM" => Some(ProductDetails {
//             name: "Smart Saver".to_string(),
//             product_type: "savings".to_string(),
//             summary: "Higher interest for regular savers with easy access and low account costs.".to_string(),
//             rate: "1.20%".to_string(),
//             minimum: "1".to_string(),
//             features: vec![
//                 "Tiered interest on balances".to_string(),
//                 "No monthly fees".to_string(),
//                 "Free card and account maintenance".to_string(),
//                 "Easy transfers and payments".to_string(),
//             ],
//         }),
//         "PL" => Some(ProductDetails {
//             name: "Personal Loan".to_string(),
//             product_type: "loan".to_string(),
//             summary: "A straight-through loan product for personal expenses with clear repayment terms.".to_string(),
//             rate: "5.88%".to_string(),
//             minimum: "0".to_string(),
//             features: vec![
//                 "Fast approval process".to_string(),
//                 "Flexible tenor options".to_string(),
//                 "Competitive interest rate".to_string(),
//                 "Digital application support".to_string(),
//             ],
//         }),
//         _ => None, // Invalid product
//     }
// }

fn account_type_label(value: &str) -> &'static str {
    match value {
        "high_yield_savings" => "RustToGold High Yield Savings Account",
        _ => "RustToGold Everyday Savings Account",
    }
}