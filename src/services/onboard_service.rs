use askama::DynTemplate;

use crate::views::{ErrorTemplate, OnboardingFormTemplate, OnboardingFormTemplate1};

pub struct ProductDetails {
    pub name: String,
    pub product_type: String,
    pub summary: String,
    pub rate: String,
    pub minimum: String,
    pub features: Vec<String>,
}

pub fn get_path_template(id: &str) -> (Option<Box<dyn DynTemplate>>, i32) {
    match id.to_lowercase().as_str() {
        "primary-contact-details" => (Some(Box::new(OnboardingFormTemplate { test: true })), 1),
        "additional-details" => (Some(Box::new(OnboardingFormTemplate1 { test: true })), 2),
        _ => (None, 0),
    }
}

pub fn get_product_details(id: &str) -> Option<ProductDetails> {
    match id.to_uppercase().as_str() {
        "XS" => Some(ProductDetails {
            name: "Everyday Savings".to_string(),
            product_type: "savings".to_string(),
            summary: "A flexible savings account for everyday spending and simple digital banking."
                .to_string(),
            rate: "0.75%".to_string(),
            minimum: "1".to_string(),
            features: vec![
                "No monthly fees".to_string(),
                "Instant debit card issuance".to_string(),
                "Online banking and mobile access".to_string(),
                "Contactless payments enabled".to_string(),
            ],
        }),
        "SM" => Some(ProductDetails {
            name: "Smart Saver".to_string(),
            product_type: "savings".to_string(),
            summary: "Higher interest for regular savers with easy access and low account costs."
                .to_string(),
            rate: "1.20%".to_string(),
            minimum: "1".to_string(),
            features: vec![
                "Tiered interest on balances".to_string(),
                "No monthly fees".to_string(),
                "Free card and account maintenance".to_string(),
                "Easy transfers and payments".to_string(),
            ],
        }),
        "PL" => Some(ProductDetails {
            name: "Personal Loan".to_string(),
            product_type: "loan".to_string(),
            summary:
                "A straight-through loan product for personal expenses with clear repayment terms."
                    .to_string(),
            rate: "5.88%".to_string(),
            minimum: "0".to_string(),
            features: vec![
                "Fast approval process".to_string(),
                "Flexible tenor options".to_string(),
                "Competitive interest rate".to_string(),
                "Digital application support".to_string(),
            ],
        }),
        _ => None, // Invalid product
    }
}
