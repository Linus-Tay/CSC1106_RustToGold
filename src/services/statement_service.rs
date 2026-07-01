// Service layer: keeps banking validation and workflow rules away from templates and SQL.

use crate::forms::StatementRequest;
use crate::models::{BankStatement, Product, StatementTransaction};
use crate::repositories::{customer_repository, product_repository, statement_repository};
use chrono::{Duration, NaiveDate, NaiveTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

const MAX_STATEMENT_DAYS: i64 = 92;

// Data carrier for the StatementPageData workflow.
pub struct StatementPageData {
    pub accounts: Vec<Product>,
    pub selected_account_id: String,
    pub start_date: String,
    pub end_date: String,
    pub transactions: Vec<StatementTransaction>,
    pub error: String,
}

struct StatementDateRange {
    start: NaiveDate,
    end: NaiveDate,
}

impl StatementDateRange {
    // Runs business logic for from request.
    fn from_request(request: &StatementRequest) -> Result<Self, String> {
        let today = Utc::now().date_naive();
        let end = parse_date_option(&request.end_date)?.unwrap_or(today);
        let start = parse_date_option(&request.start_date)?.unwrap_or(end - Duration::days(30));

        if start > end {
            return Err("Start date cannot be after end date.".to_string());
        }

        if end.signed_duration_since(start).num_days() > MAX_STATEMENT_DAYS {
            return Err("Statement date range cannot exceed 3 months.".to_string());
        }

        if end > today {
            return Err("End date cannot be in the future.".to_string());
        }

        Ok(Self { start, end })
    }

    // Runs business logic for start at.
    fn start_at(&self) -> chrono::NaiveDateTime {
        self.start.and_time(midnight())
    }

    // Runs business logic for end exclusive.
    fn end_exclusive(&self) -> chrono::NaiveDateTime {
        (self.end + Duration::days(1)).and_time(midnight())
    }

    // Runs business logic for start input.
    fn start_input(&self) -> String {
        self.start.format("%Y-%m-%d").to_string()
    }

    // Runs business logic for end input.
    fn end_input(&self) -> String {
        self.end.format("%Y-%m-%d").to_string()
    }
}

// Loads statement page data and applies page-level business rules.
pub async fn load_statement_page(
    db: &PgPool,
    customer_id: Uuid,
    request: StatementRequest,
) -> Result<StatementPageData, String> {
    let accounts = product_repository::list_active_products_by_customer(db, &customer_id)
        .await
        .map_err(|_| "Could not load your active accounts.".to_string())?;

    if accounts.is_empty() {
        return Ok(StatementPageData {
            accounts,
            selected_account_id: String::new(),
            start_date: String::new(),
            end_date: String::new(),
            transactions: Vec::new(),
            error: String::new(),
        });
    }

    let range = match StatementDateRange::from_request(&request) {
        Ok(range) => range,
        Err(error) => {
            let today = Utc::now().date_naive();
            let fallback = StatementDateRange {
                start: today - Duration::days(30),
                end: today,
            };
            return Ok(StatementPageData {
                selected_account_id: accounts[0].id.to_string(),
                start_date: fallback.start_input(),
                end_date: fallback.end_input(),
                accounts,
                transactions: Vec::new(),
                error,
            });
        }
    };

    let account = pick_account(db, customer_id, &accounts, request.account_id.as_deref()).await?;
    let transactions = statement_repository::find_transactions_for_product_in_range(
        db,
        account.id,
        range.start_at(),
        range.end_exclusive(),
    )
    .await
    .map_err(|_| "Could not load transactions for this statement.".to_string())?;

    Ok(StatementPageData {
        selected_account_id: account.id.to_string(),
        start_date: range.start_input(),
        end_date: range.end_input(),
        accounts,
        transactions,
        error: String::new(),
    })
}

/// Builds the statement data after validating ownership and date range.
pub async fn build_bank_statement(
    db: &PgPool,
    customer_id: Uuid,
    request: StatementRequest,
) -> Result<BankStatement, String> {
    let range = StatementDateRange::from_request(&request)?;
    let accounts = product_repository::list_active_products_by_customer(db, &customer_id)
        .await
        .map_err(|_| "Could not load your active accounts.".to_string())?;

    if accounts.is_empty() {
        return Err("No active account was found for statement download.".to_string());
    }

    let account = pick_account(db, customer_id, &accounts, request.account_id.as_deref()).await?;
    let customer = customer_repository::get_customer_by_id(db, &customer_id)
        .await
        .map_err(|_| "Could not load your customer profile.".to_string())?;

    let transactions = statement_repository::find_transactions_for_product_in_range(
        db,
        account.id,
        range.start_at(),
        range.end_exclusive(),
    )
    .await
    .map_err(|_| "Could not load transactions for this statement.".to_string())?;

    let today = Utc::now().date_naive();
    let opening_balance_cents = match statement_repository::find_latest_balance_before(db, account.id, range.start_at()).await {
        Ok(Some(balance)) => balance,
        Ok(None) => transactions
            .first()
            .map(|transaction| transaction.balance_after_cents - transaction.signed_amount_cents())
            .unwrap_or_else(|| if range.end >= today { account.balance_cents } else { 0 }),
        Err(_) => return Err("Could not calculate opening balance.".to_string()),
    };

    let closing_balance_cents = match statement_repository::find_latest_balance_before(db, account.id, range.end_exclusive()).await {
        Ok(Some(balance)) => balance,
        Ok(None) => transactions
            .last()
            .map(|transaction| transaction.balance_after_cents)
            .unwrap_or_else(|| if range.end >= today { account.balance_cents } else { opening_balance_cents }),
        Err(_) => return Err("Could not calculate closing balance.".to_string()),
    };

    Ok(BankStatement {
        customer_name: customer.full_name,
        customer_email: customer.email,
        account,
        start_date: range.start,
        end_date: range.end,
        generated_at: Utc::now().naive_utc(),
        opening_balance_cents,
        closing_balance_cents,
        transactions,
    })
}

// Runs business logic for statement pdf filename.
pub fn statement_pdf_filename(statement: &BankStatement) -> String {
    let account = statement.account_number().replace('-', "");
    format!(
        "RustToGold_statement_{}_{}_to_{}.pdf",
        account,
        statement.start_date.format("%Y%m%d"),
        statement.end_date.format("%Y%m%d")
    )
}

/// Renders a lightweight PDF so the statement can be downloaded without extra services.
pub fn render_statement_pdf(statement: &BankStatement) -> Vec<u8> {
    StatementPdfBuilder::new(statement).build()
}

// Runs business logic for pick account.
async fn pick_account(
    db: &PgPool,
    customer_id: Uuid,
    accounts: &[Product],
    requested_account_id: Option<&str>,
) -> Result<Product, String> {
    let Some(raw_account_id) = requested_account_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return accounts
            .first()
            .cloned()
            .ok_or_else(|| "No active account was found.".to_string());
    };

    let account_id = Uuid::parse_str(raw_account_id)
        .map_err(|_| "Choose a valid account for the statement.".to_string())?;

    product_repository::get_active_product_for_customer_by_id(db, customer_id, account_id)
        .await
        .map_err(|_| "Choose an active account that belongs to you.".to_string())
}

// Runs business logic for midnight.
fn midnight() -> NaiveTime {
    NaiveTime::from_hms_opt(0, 0, 0).expect("midnight should always be valid")
}

// Parses date option from form input into a safer internal value.
fn parse_date_option(value: &Option<String>) -> Result<Option<NaiveDate>, String> {
    let Some(value) = value.as_deref().map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| "Use a valid statement date.".to_string())
}

struct StatementPdfBuilder<'a> {
    statement: &'a BankStatement,
}

impl<'a> StatementPdfBuilder<'a> {
    // Runs business logic for new.
    fn new(statement: &'a BankStatement) -> Self {
        Self { statement }
    }

    // Runs business logic for build.
    fn build(&self) -> Vec<u8> {
        let mut pages: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut y = 790.0;

        self.draw_header(&mut current, &mut y);
        self.draw_summary(&mut current, &mut y);
        self.draw_table_header(&mut current, &mut y);

        if self.statement.transactions.is_empty() {
            draw_text(&mut current, 50.0, y, 10, "No transactions found for this date range.");
            y -= 18.0;
        } else {
            for transaction in &self.statement.transactions {
                if y < 90.0 {
                    pages.push(current);
                    current = String::new();
                    y = 790.0;
                    self.draw_page_title(&mut current, &mut y);
                    self.draw_table_header(&mut current, &mut y);
                }

                self.draw_transaction_row(&mut current, &mut y, transaction);
            }
        }

        self.draw_footer(&mut current);
        pages.push(current);
        build_pdf_document(pages)
    }

    // Runs business logic for draw header.
    fn draw_header(&self, page: &mut String, y: &mut f32) {
        draw_text(page, 50.0, *y, 20, "RustToGold Bank Statement");
        draw_text(page, 50.0, *y - 24.0, 10, "Generated from the RustToGold customer banking portal.");
        draw_text(page, 400.0, *y, 10, &format!("Generated: {}", self.statement.generated_at_display()));
        *y -= 58.0;
    }

    // Runs business logic for draw page title.
    fn draw_page_title(&self, page: &mut String, y: &mut f32) {
        draw_text(page, 50.0, *y, 14, "RustToGold Bank Statement continued");
        draw_text(page, 50.0, *y - 18.0, 9, &format!("Account: {}", self.statement.account_number()));
        *y -= 42.0;
    }

    // Runs business logic for draw summary.
    fn draw_summary(&self, page: &mut String, y: &mut f32) {
        let rows = [
            format!("Customer: {}", self.statement.customer_name),
            format!("Email: {}", self.statement.customer_email),
            format!("Account: {}", self.statement.account_number()),
            format!("Product: {}", self.statement.product_name()),
            format!("Period: {}", self.statement.period_display()),
            format!("Opening balance: {}", self.statement.opening_balance_display()),
            format!("Closing balance: {}", self.statement.closing_balance_display()),
        ];

        for row in rows {
            draw_text(page, 50.0, *y, 10, &row);
            *y -= 15.0;
        }

        *y -= 12.0;
    }

    // Runs business logic for draw table header.
    fn draw_table_header(&self, page: &mut String, y: &mut f32) {
        draw_text(page, 50.0, *y, 9, "Date");
        draw_text(page, 120.0, *y, 9, "Description");
        draw_text(page, 330.0, *y, 9, "Debit");
        draw_text(page, 405.0, *y, 9, "Credit");
        draw_text(page, 480.0, *y, 9, "Balance");
        *y -= 14.0;
        draw_line(page, 50.0, *y, 545.0, *y);
        *y -= 16.0;
    }

    // Runs business logic for draw transaction row.
    fn draw_transaction_row(&self, page: &mut String, y: &mut f32, transaction: &StatementTransaction) {
        let description = format!("{} - {}", transaction.transaction_type_display(), transaction.description_display());
        draw_text(page, 50.0, *y, 8, &transaction.date_display());
        draw_text(page, 120.0, *y, 8, &truncate(&description, 44));
        draw_text(page, 330.0, *y, 8, &transaction.debit_display());
        draw_text(page, 405.0, *y, 8, &transaction.credit_display());
        draw_text(page, 480.0, *y, 8, &transaction.balance_after_display());
        *y -= 16.0;
    }

    // Runs business logic for draw footer.
    fn draw_footer(&self, page: &mut String) {
        draw_line(page, 50.0, 58.0, 545.0, 58.0);
        draw_text(page, 50.0, 42.0, 8, "This PDF is generated by RustToGold and is not an official bank statement.");
    }
}

// Runs business logic for draw text.
fn draw_text(page: &mut String, x: f32, y: f32, size: i32, text: &str) {
    page.push_str(&format!(
        "BT /F1 {} Tf {:.2} {:.2} Td ({}) Tj ET\n",
        size,
        x,
        y,
        escape_pdf_text(text)
    ));
}

// Runs business logic for draw line.
fn draw_line(page: &mut String, x1: f32, y1: f32, x2: f32, y2: f32) {
    page.push_str(&format!("{:.2} {:.2} m {:.2} {:.2} l S\n", x1, y1, x2, y2));
}

// Runs business logic for truncate.
fn truncate(value: &str, limit: usize) -> String {
    let mut output = String::new();
    for character in value.chars().take(limit) {
        output.push(character);
    }
    if value.chars().count() > limit {
        output.push_str("...");
    }
    output
}

// Runs business logic for escape pdf text.
fn escape_pdf_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

// Runs business logic for build pdf document.
fn build_pdf_document(pages: Vec<String>) -> Vec<u8> {
    let mut objects: Vec<String> = Vec::new();
    let font_id = 3;
    let mut kids = Vec::new();

    objects.push("<< /Type /Catalog /Pages 2 0 R >>".to_string());
    objects.push(String::new());
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string());

    for (index, content) in pages.iter().enumerate() {
        let page_id = 4 + index * 2;
        let content_id = page_id + 1;
        kids.push(format!("{} 0 R", page_id));
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 {} 0 R >> >> /Contents {} 0 R >>",
            font_id,
            content_id
        ));
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}endstream",
            content.as_bytes().len(),
            content
        ));
    }

    objects[1] = format!(
        "<< /Type /Pages /Kids [{}] /Count {} >>",
        kids.join(" "),
        pages.len()
    );

    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n");
    let mut offsets = vec![0usize];

    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", index + 1, object).as_bytes());
    }

    let xref_start = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{:010} 00000 n \n", offset).as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
            objects.len() + 1,
            xref_start
        )
        .as_bytes(),
    );

    pdf
}
