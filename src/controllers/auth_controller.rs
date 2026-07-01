// Controller layer: handles HTTP/session flow and delegates business rules to services.

use crate::controllers::session_guard::{
    admin_session_user_id, clear_admin_session, clear_customer_session, customer_session_user_id,
    redirect, store_admin_session, store_customer_session,
};
use crate::forms::LoginForm;
use crate::services::{self, authenticate_device};
use crate::views::{AdminLoginTemplate, ErrorTemplate, LoginTemplate, render};
use crate::AppState;
use actix_session::Session;
use actix_web::http::Error;
use actix_web::{web, HttpResponse, Result, HttpRequest};
use sha2::{Sha256, Digest};

// Renders the login page screen with data prepared by the service layer.
pub async fn login_page(session: Session) -> Result<HttpResponse> {
    if customer_session_user_id(&session).is_some() {
        return Ok(redirect("/customer/dashboard"));
    }

    render(LoginTemplate {
        error: String::new(),
        has_error: false,
    })
}

// Renders the admin login page screen with data prepared by the service layer.
pub async fn admin_login_page(session: Session) -> Result<HttpResponse> {
    if admin_session_user_id(&session).is_some() {
        return Ok(redirect("/admin/dashboard"));
    }

    render(AdminLoginTemplate {
        error: String::new(),
        has_error: false,
    })
}

// Handles login session flow.
pub async fn login(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<LoginForm>,
    req: HttpRequest // <-- FIXED: Changed from HttpResponse to HttpRequest
) -> Result<HttpResponse> {
    match services::authenticate_user(&data.db, form.into_inner()).await {
        Ok(user) if user.is_customer() => {
            if let Some(cookie) = req.cookie("device_id") {
                let raw_token = cookie.value();
                if let Ok(_device) = authenticate_device(&data.db, raw_token).await {
                    store_customer_session(&session, &user)?;
                    return Ok(redirect("/customer/dashboard")); 
                }
            }

            match services::generate_and_send_2fa(&data.db, &user.id).await {
                Ok(()) => (),
                Err(e) => {
                    return render(ErrorTemplate)
                }
            };

            session.insert("pending_2fa_user_id", user.id.to_string())?;
            Ok(redirect("/customer/verify-2fa"))
        }
        Ok(_) => render(LoginTemplate {
            error: "Use the admin login page for staff access.".to_string(),
            has_error: true,
        }),
        Err(error) => render(LoginTemplate {
            error,
            has_error: true,
        }),
    }
}

// Handles login session flow.
pub async fn verify_2fa(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<LoginForm>,
    req: HttpRequest // <-- FIXED: Changed from HttpResponse to HttpRequest
) -> Result<HttpResponse> {
    match services::authenticate_user(&data.db, form.into_inner()).await {
        Ok(user) if user.is_customer() => {
            if let Some(cookie) = req.cookie("device_id") {
                let raw_token = cookie.value();
                if let Ok(_device) = authenticate_device(&data.db, raw_token).await {
                    store_customer_session(&session, &user)?;
                    return Ok(redirect("/customer/dashboard")); 
                }
            }

            session.insert("pending_2fa_user_id", user.id.to_string())?;
            Ok(redirect("/customer/verify-2fa"))
        }
        Ok(_) => render(LoginTemplate {
            error: "Use the admin login page for staff access.".to_string(),
            has_error: true,
        }),
        Err(error) => render(LoginTemplate {
            error,
            has_error: true,
        }),
    }
}

// Handles admin login session flow.
pub async fn admin_login(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<LoginForm>,
) -> Result<HttpResponse> {
    match services::authenticate_user(&data.db, form.into_inner()).await {
        Ok(user) if user.is_staff_or_admin() => {
            store_admin_session(&session, &user)?;
            Ok(redirect("/admin/dashboard"))
        }
        Ok(_) => render(AdminLoginTemplate {
            error: "This login is only for staff and admin users.".to_string(),
            has_error: true,
        }),
        Err(error) => render(AdminLoginTemplate {
            error,
            has_error: true,
        }),
    }
}

// Handles logout session flow.
pub async fn logout(session: Session) -> Result<HttpResponse> {
    clear_customer_session(&session);
    Ok(redirect("/"))
}

// Handles admin logout session flow.
pub async fn admin_logout(session: Session) -> Result<HttpResponse> {
    clear_admin_session(&session);
    Ok(redirect("/admin/login"))
}