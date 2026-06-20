use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateFixedDepositForm {
    pub plan_id: String,
    pub amount: String,
}

#[derive(Debug, Deserialize)]
pub struct FixedDepositPlanForm {
    pub name: String,
    pub duration_months: String,
    pub interest_rate: String,
    pub minimum_amount: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct FixedDepositMessageQuery {
    pub created: Option<String>,
    pub withdrawn: Option<String>,
    pub paid_out: Option<String>,
    pub updated: Option<String>,
}
