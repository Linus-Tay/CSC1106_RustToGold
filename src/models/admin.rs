use super::Money;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AdminDashboardSummary {
    pub pending_signup_count: i64,
    pub pending_personal_loan_count: i64,
    pub pending_home_loan_count: i64,
    pub active_fixed_deposit_count: i64,
    pub total_customer_count: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct AdminCustomerApplication {
    pub customer_id: Uuid,
    pub user_id: i64,
    pub full_name: String,
    pub email: String,
    pub phone_number: String,
    pub nric: String,
    pub date_of_birth: NaiveDate,
    pub gender: String,
    pub nationality: String,
    pub residency: String,
    pub race: Option<String>,
    pub residential_address: String,
    pub mailing_address: Option<String>,
    pub preferred_contact: Option<String>,
    pub employment_status: String,
    pub occupation: Option<String>,
    pub employer_name: Option<String>,
    pub industry: Option<String>,
    pub monthly_income_range: Option<String>,
    pub kyc_status: String,
    pub user_status: String,
    pub account_number: Option<String>,
    pub selected_account_type: Option<String>,
    pub product_type: Option<String>,
    pub product_status: Option<String>,
    pub account_balance_cents: Option<i64>,
    pub account_created_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl AdminCustomerApplication {
    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y").to_string()
    }

    pub fn date_of_birth_display(&self) -> String {
        self.date_of_birth.format("%d %b %Y").to_string()
    }

    pub fn age_display(&self) -> String {
        let today = Utc::now().date_naive();
        let mut age = today.year() - self.date_of_birth.year();
        if (today.month(), today.day()) < (self.date_of_birth.month(), self.date_of_birth.day()) {
            age -= 1;
        }
        format!("{} years old", age)
    }

    pub fn status_display(&self) -> String {
        title_status(&self.kyc_status, &[("pending", "Pending Review"), ("approved", "Approved"), ("rejected", "Rejected")])
    }

    pub fn user_status_display(&self) -> String {
        title_case(&self.user_status)
    }

    pub fn account_number_display(&self) -> String {
        option_display(&self.account_number, "Not created")
    }

    pub fn account_type_display(&self) -> String {
        self.selected_account_type
            .as_deref()
            .map(title_case)
            .unwrap_or_else(|| "Not selected".to_string())
    }

    pub fn product_type_display(&self) -> String {
        self.product_type
            .as_deref()
            .map(title_case)
            .unwrap_or_else(|| "Not created".to_string())
    }

    pub fn product_status_display(&self) -> String {
        self.product_status
            .as_deref()
            .map(title_case)
            .unwrap_or_else(|| "No product".to_string())
    }

    pub fn account_balance_display(&self) -> String {
        self.account_balance_cents
            .map(|value| Money::from_cents(value).display())
            .unwrap_or_else(|| "$0.00".to_string())
    }

    pub fn account_created_display(&self) -> String {
        self.account_created_at
            .map(|value| value.format("%d %b %Y").to_string())
            .unwrap_or_else(|| "Not created".to_string())
    }

    pub fn race_display(&self) -> String {
        option_display(&self.race, "Not collected")
    }

    pub fn mailing_address_display(&self) -> String {
        option_display(&self.mailing_address, "Same as residential address")
    }

    pub fn preferred_contact_display(&self) -> String {
        self.preferred_contact
            .as_deref()
            .map(title_case)
            .unwrap_or_else(|| "Not specified".to_string())
    }

    pub fn employment_status_display(&self) -> String {
        title_case(&self.employment_status)
    }

    pub fn occupation_display(&self) -> String {
        option_display(&self.occupation, "Not provided")
    }

    pub fn employer_display(&self) -> String {
        option_display(&self.employer_name, "Not provided")
    }

    pub fn industry_display(&self) -> String {
        option_display(&self.industry, "Not provided")
    }

    pub fn income_display(&self) -> String {
        option_display(&self.monthly_income_range, "Not provided")
    }

    pub fn review_note(&self) -> &'static str {
        if self.kyc_status == "pending" {
            "Check identity, contactability, residency, employment and selected product before approval."
        } else {
            "Application has already been reviewed."
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct AdminPersonalLoanRecord {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: String,
    pub customer_nric: String,
    pub kyc_status: String,
    pub employment_status: String,
    pub occupation: Option<String>,
    pub employer_name: Option<String>,
    pub monthly_income_range: Option<String>,
    pub account_number: Option<String>,
    pub account_balance_cents: Option<i64>,
    pub account_status: Option<String>,
    pub purpose: String,
    pub principal_cents: i64,
    pub annual_rate_bps: i32,
    pub term_months: i32,
    pub monthly_payment_cents: i64,
    pub outstanding_cents: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl AdminPersonalLoanRecord {
    pub fn principal_display(&self) -> String {
        Money::from_cents(self.principal_cents).display()
    }

    pub fn monthly_payment_display(&self) -> String {
        Money::from_cents(self.monthly_payment_cents).display()
    }

    pub fn outstanding_display(&self) -> String {
        Money::from_cents(self.outstanding_cents).display()
    }

    pub fn account_balance_display(&self) -> String {
        self.account_balance_cents
            .map(|value| Money::from_cents(value).display())
            .unwrap_or_else(|| "$0.00".to_string())
    }

    pub fn account_number_display(&self) -> String {
        option_display(&self.account_number, "No active account")
    }

    pub fn account_status_display(&self) -> String {
        self.account_status
            .as_deref()
            .map(title_case)
            .unwrap_or_else(|| "Unknown".to_string())
    }

    pub fn rate_display(&self) -> String {
        format!("{:.2}%", self.annual_rate_bps as f64 / 100.0)
    }

    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y").to_string()
    }

    pub fn status_display(&self) -> String {
        title_status(&self.status, &[("pending", "Pending Review"), ("active", "Active"), ("rejected", "Rejected"), ("fully_paid", "Fully Paid"), ("cancelled", "Cancelled")])
    }

    pub fn kyc_status_display(&self) -> String {
        title_status(&self.kyc_status, &[("pending", "Pending Review"), ("approved", "Approved"), ("rejected", "Rejected")])
    }

    pub fn employment_status_display(&self) -> String {
        title_case(&self.employment_status)
    }

    pub fn occupation_display(&self) -> String {
        option_display(&self.occupation, "Not provided")
    }

    pub fn employer_display(&self) -> String {
        option_display(&self.employer_name, "Not provided")
    }

    pub fn income_display(&self) -> String {
        option_display(&self.monthly_income_range, "Not provided")
    }

    pub fn term_display(&self) -> String {
        format!("{} months", self.term_months)
    }

    pub fn review_note(&self) -> &'static str {
        if self.kyc_status != "approved" {
            "Customer KYC is not approved. Review carefully before loan approval."
        } else if self.status == "pending" {
            "Confirm purpose, income band, repayment amount and account status before approval."
        } else {
            "Loan has already been reviewed."
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct AdminHomeLoanRecord {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: String,
    pub customer_nric: String,
    pub kyc_status: String,
    pub employment_status: String,
    pub occupation: Option<String>,
    pub employer_name: Option<String>,
    pub monthly_income_range: Option<String>,
    pub account_number: Option<String>,
    pub account_balance_cents: Option<i64>,
    pub account_status: Option<String>,
    pub property_type: String,
    pub property_value_cents: i64,
    pub down_payment_cents: i64,
    pub loan_amount_cents: i64,
    pub annual_rate_bps: i32,
    pub term_years: i32,
    pub monthly_payment_cents: i64,
    pub outstanding_cents: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl AdminHomeLoanRecord {
    pub fn property_value_display(&self) -> String {
        Money::from_cents(self.property_value_cents).display()
    }

    pub fn down_payment_display(&self) -> String {
        Money::from_cents(self.down_payment_cents).display()
    }

    pub fn loan_amount_display(&self) -> String {
        Money::from_cents(self.loan_amount_cents).display()
    }

    pub fn monthly_payment_display(&self) -> String {
        Money::from_cents(self.monthly_payment_cents).display()
    }

    pub fn outstanding_display(&self) -> String {
        Money::from_cents(self.outstanding_cents).display()
    }

    pub fn account_balance_display(&self) -> String {
        self.account_balance_cents
            .map(|value| Money::from_cents(value).display())
            .unwrap_or_else(|| "$0.00".to_string())
    }

    pub fn account_number_display(&self) -> String {
        option_display(&self.account_number, "No active account")
    }

    pub fn account_status_display(&self) -> String {
        self.account_status
            .as_deref()
            .map(title_case)
            .unwrap_or_else(|| "Unknown".to_string())
    }

    pub fn rate_display(&self) -> String {
        format!("{:.2}%", self.annual_rate_bps as f64 / 100.0)
    }

    pub fn created_at_display(&self) -> String {
        self.created_at.format("%d %b %Y").to_string()
    }

    pub fn status_display(&self) -> String {
        title_status(&self.status, &[("pending", "Pending Review"), ("approved", "Approved"), ("rejected", "Rejected"), ("fully_paid", "Fully Paid")])
    }

    pub fn kyc_status_display(&self) -> String {
        title_status(&self.kyc_status, &[("pending", "Pending Review"), ("approved", "Approved"), ("rejected", "Rejected")])
    }

    pub fn employment_status_display(&self) -> String {
        title_case(&self.employment_status)
    }

    pub fn occupation_display(&self) -> String {
        option_display(&self.occupation, "Not provided")
    }

    pub fn employer_display(&self) -> String {
        option_display(&self.employer_name, "Not provided")
    }

    pub fn income_display(&self) -> String {
        option_display(&self.monthly_income_range, "Not provided")
    }

    pub fn loan_to_value_display(&self) -> String {
        ratio_percent(self.loan_amount_cents, self.property_value_cents)
    }

    pub fn down_payment_percent_display(&self) -> String {
        ratio_percent(self.down_payment_cents, self.property_value_cents)
    }

    pub fn term_display(&self) -> String {
        format!("{} years", self.term_years)
    }

    pub fn review_note(&self) -> &'static str {
        if self.kyc_status != "approved" {
            "Customer KYC is not approved. Check identity and account status before approval."
        } else if self.status == "pending" {
            "Review property value, down payment, LTV, income band and monthly repayment before approval."
        } else {
            "Application has already been reviewed."
        }
    }
}

fn option_display(value: &Option<String>, fallback: &str) -> String {
    value
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| fallback.to_string())
}

fn title_case(value: &str) -> String {
    value
        .replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_status(value: &str, labels: &[(&str, &str)]) -> String {
    labels
        .iter()
        .find(|(key, _)| *key == value)
        .map(|(_, label)| (*label).to_string())
        .unwrap_or_else(|| title_case(value))
}

fn ratio_percent(numerator: i64, denominator: i64) -> String {
    if denominator <= 0 {
        return "0.0%".to_string();
    }
    format!("{:.1}%", numerator as f64 / denominator as f64 * 100.0)
}
