use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateFixedDepositForm {
    pub plan_id: i64,
    pub amount: String,
}

#[derive(Debug, Deserialize)]
pub struct FixedDepositPlanForm {
    pub plan_name: String,
    pub tenure_months: i32,
    pub annual_rate_bps: i32,
    pub minimum_amount: String,
    pub is_active: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FixedDepositMessageQuery {
    pub created: Option<String>,
    pub withdrawn: Option<String>,
    pub paid_out: Option<String>,
    pub updated: Option<String>,
}
