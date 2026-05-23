use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateFixedDepositForm {
    pub plan_id: i64,
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
