#[derive(Debug, Clone)]
pub struct ProductSummary {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub summary: &'static str,
    pub rate_or_pricing: &'static str,
    pub requirement: &'static str,
    pub features: &'static [&'static str],
}

pub fn find_product(raw_product_id: &str) -> Option<ProductSummary> {
    let normalised = raw_product_id.trim().to_ascii_uppercase();

    match normalised.as_str() {
        "ESA" | "EVERYDAY_SAVINGS" => Some(ProductSummary {
            id: "ESA",
            name: "Everyday Savings Account",
            category: "Bank Account",
            summary: "A daily-use savings account for salary crediting, deposits, balance checks, online banking, and linked debit card access.",
            rate_or_pricing: "Standard savings interest",
            requirement: "Master KYC and customer profile required",
            features: &[
                "Creates a deposit ledger and account number",
                "Includes debit card access as part of the account journey",
                "Suitable as the default account for everyday banking",
                "Can support transfers, statements, and transaction history",
            ],
        }),
        "SSA" | "SMART_SAVER" | "SM" => Some(ProductSummary {
            id: "SSA",
            name: "Smart Saver Account",
            category: "Bank Account",
            summary: "A savings account positioned for goal-based saving, tiered interest messaging, and monthly balance tracking.",
            rate_or_pricing: "Tiered savings interest",
            requirement: "Master KYC and customer profile required",
            features: &[
                "Creates a deposit ledger and account number",
                "Supports saver-focused product messaging",
                "Can link debit access if the customer requests it",
                "Useful for differentiating account products beyond normal savings",
            ],
        }),
        "MCA" | "MULTI_CURRENCY" => Some(ProductSummary {
            id: "MCA",
            name: "Multi-Currency Account",
            category: "Bank Account",
            summary: "An account option for customers who want selected foreign-currency balances alongside normal SGD banking.",
            rate_or_pricing: "Currency-dependent",
            requirement: "Master KYC and customer profile required",
            features: &[
                "Creates a deposit ledger with multi-currency positioning",
                "Supports travel and overseas-study use cases",
                "Can be linked to the Foreign Exchange Card later",
                "Keeps the FX account flow separate from normal savings",
            ],
        }),
        "FD" | "FIXED_DEPOSIT" => Some(ProductSummary {
            id: "FD",
            name: "Fixed Deposit",
            category: "Deposit Product",
            summary: "A term deposit placement where the customer selects amount, tenure, projected return, and maturity instruction.",
            rate_or_pricing: "Tenure-dependent rate",
            requirement: "Source account or funding instruction required",
            features: &[
                "Captures placement amount and tenure",
                "Shows projected return before confirmation",
                "Stores maturity and payout instruction",
                "Branches into fixed-deposit placement logic rather than normal account opening",
            ],
        }),
        "PL" | "PERSONAL_LOAN" => Some(ProductSummary {
            id: "PL",
            name: "Personal Loan",
            category: "Lending Product",
            summary: "A loan application flow for requested amount, purpose, tenure, income profile, review status, and repayment estimate.",
            rate_or_pricing: "Subject to approval",
            requirement: "Income and eligibility review required",
            features: &[
                "Does not require an existing bank account before applying",
                "Creates a lending application after customer profile creation",
                "Supports pending, approved, rejected, and active states",
                "Can later disburse into a selected account or external instruction",
            ],
        }),
        "FCC" | "FLEX_CREDIT" | "CC" => Some(ProductSummary {
            id: "FCC",
            name: "Flex Credit Card",
            category: "Credit Card",
            summary: "A standalone credit card application that branches into credit review, card limit assignment, billing, and repayment logic.",
            rate_or_pricing: "Subject to approval",
            requirement: "Income and credit assessment required",
            features: &[
                "Does not require an existing savings account before applying",
                "Branches into credit-card workflow after master KYC",
                "Supports credit limit, statement, billing, and repayment concepts",
                "Keeps debit-card logic separate from credit-card logic",
            ],
        }),
        "FXC" | "FOREIGN_EXCHANGE_CARD" | "FX_CARD" => Some(ProductSummary {
            id: "FXC",
            name: "Foreign Exchange Card",
            category: "Card Product",
            summary: "A travel-focused card concept for selected currency wallets, overseas spending visibility, and exchange-rate-aware transaction records.",
            rate_or_pricing: "Currency conversion applies",
            requirement: "Customer profile required; multi-currency account linkage optional",
            features: &[
                "Useful for travel and overseas-study customers",
                "Can be linked to a multi-currency account",
                "Tracks overseas spending and currency wallet concepts",
                "Distinct from both debit-card and credit-card workflows",
            ],
        }),
        _ => None,
    }
}
