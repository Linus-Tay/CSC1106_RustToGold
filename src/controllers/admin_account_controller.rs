use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::{redirect, require_staff_or_admin, require_admin};
use crate::services;
use crate::services::AuditContext;
use crate::views::render;
use crate::views::templates::{AdminAccountsTemplate, AdminTransactionsTemplate};
use crate::AppState;
use crate::repositories::{account_repository, fixed_deposit_repository, staff_repository, transaction_repository, audit_log_repository, admin_transaction_repository};
use actix_session::Session;
use actix_web::{web, HttpRequest, HttpResponse, Result};

/*pub async fn admin_dashboard_page(
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    if let Err(response) = require_staff_or_admin(&data, &session).await {
        return Ok(response);
    }

    let db = &data.db;

    let total_deposits = account_repository::count_accounts(db, None)
        .await
        .unwrap_or(0);

    let active_plans = fixed_deposit_repository::count_active_plans(db)
        .await
        .unwrap_or(0);

    let staff_count = staff_repository::count_staff(db)
        .await
        .unwrap_or(0);

    let transaction_count = transaction_repository::count_transactions(db)
        .await
        .unwrap_or(0);

    let recent_audit = audit_log_repository::list_recent_audit(db, 5)
        .await
        .unwrap_or_default();

    render(AdminDashboardTemplate {
        total_deposits,
        active_plans,
        staff_count,
        transaction_count,
        has_recent_audit: !recent_audit.is_empty(),
        recent_audit,
    })
}*/

/// GET /admin/accounts?status=&page=
pub async fn admin_accounts_page(
    data: web::Data<AppState>,
    session: Session,
    query: web::Query<AdminAccountQuery>,
) -> Result<HttpResponse> {
    if let Err(response) = require_staff_or_admin(&data, &session).await {
        return Ok(response);
    }

    let page = query.page.unwrap_or(1);
    let status = query.status.clone();

    match services::load_admin_accounts(&data.db, status.clone(), page).await {
        Ok(result) => render(AdminAccountsTemplate {
            accounts: result.accounts,
            has_accounts: result.has_accounts,
            total_count: result.total_count,
            page: result.page,
            total_pages: result.total_pages,
            filter_status: status.unwrap_or_default(),
            success: query.success.clone().unwrap_or_default(),
            has_success: query.success.is_some(),
            error: String::new(),
            has_error: false,
        }),
        Err(message) => render_error("Accounts unavailable", message),
    }
}

/// POST /admin/accounts/{id}/approve
pub async fn approve_account(
    data: web::Data<AppState>,
    session: Session,
    req: HttpRequest,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let actor = match require_staff_or_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let ctx = build_ctx(&actor.id, &req);

    match services::update_account_status(&data.db, &ctx, path.into_inner(), "active").await {
        Ok(_) => Ok(redirect("/admin/accounts?status=pending&success=approved")),
        Err(message) => render_error("Could not approve account", message),
    }
}

/// POST /admin/accounts/{id}/freeze
pub async fn freeze_account(
    data: web::Data<AppState>,
    session: Session,
    req: HttpRequest,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let actor = match require_staff_or_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let ctx = build_ctx(&actor.id, &req);

    match services::update_account_status(&data.db, &ctx, path.into_inner(), "frozen").await {
        Ok(_) => Ok(redirect("/admin/accounts?success=frozen")),
        Err(message) => render_error("Could not freeze account", message),
    }
}

/// POST /admin/accounts/{id}/close
pub async fn close_account(
    data: web::Data<AppState>,
    session: Session,
    req: HttpRequest,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let actor = match require_staff_or_admin(&data, &session).await {
        Ok(user) => user,
        Err(response) => return Ok(response),
    };

    let ctx = build_ctx(&actor.id, &req);

    match services::update_account_status(&data.db, &ctx, path.into_inner(), "closed").await {
        Ok(_) => Ok(redirect("/admin/accounts?success=closed")),
        Err(message) => render_error("Could not close account", message),
    }
}

fn build_ctx(user_id: &i64, req: &HttpRequest) -> AuditContext {
    AuditContext {
        actor_user_id: Some(*user_id),
        ip_address: req.peer_addr().map(|a| a.ip().to_string()),
        user_agent: req
            .headers()
            .get("User-Agent")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct AdminAccountQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub success: Option<String>,
}

pub async fn admin_transactions_page(
    data: web::Data<AppState>,
    session: Session,
    query: web::Query<AdminTransactionQuery>,
) -> Result<HttpResponse> {
    if let Err(response) = require_admin(&data, &session).await {
        return Ok(response);
    }

    let page = query.page.unwrap_or(1);

    match services::load_admin_transactions(
        &data.db,
        query.transaction_type.clone(),
        query.user_id.clone(),
        query.account_id.clone(),
        page,
    ).await {
        Ok(result) => render(AdminTransactionsTemplate {
            transactions: result.transactions,
            has_transactions: result.has_transactions,
            total_count: result.total_count,
            page: result.page,
            total_pages: result.total_pages,
            filter_transaction_type: query.transaction_type.clone().unwrap_or_default(),
            filter_user_id: query.user_id.clone().unwrap_or_default(),
            filter_account_id: query.account_id.clone().unwrap_or_default(),
        }),
        Err(message) => render_error("Transactions unavailable", message),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct AdminTransactionQuery {
    pub transaction_type: Option<String>,
    pub user_id: Option<String>,
    pub account_id: Option<String>,
    pub page: Option<i64>,
}