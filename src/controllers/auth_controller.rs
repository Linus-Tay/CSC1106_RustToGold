use crate::controllers::session_guard::{redirect, session_user_id};
use crate::forms::{LoginForm, SignupForm};
use crate::services;
use crate::views::{render, LoginTemplate, SignupTemplate};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};

pub async fn login_page(session: Session) -> Result<HttpResponse> {
    if session_user_id(&session).is_some() {
        return Ok(redirect_after_login(&session));
    }

    render(LoginTemplate {
        error: String::new(),
        has_error: false,
    })
}

pub async fn login(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<LoginForm>,
) -> Result<HttpResponse> {
    match services::authenticate_user(&data.db, form.into_inner()).await {
        Ok(user) => {
            session.insert("user_id", user.id)?;
            session.insert("role", user.role.clone())?;

            if user.role == "admin" {
                Ok(redirect("/admin/fixed-deposits"))
            } else {
                Ok(redirect("/customer/dashboard"))
            }
        }
        Err(error) => render(LoginTemplate {
            error,
            has_error: true,
        }),
    }
}

pub async fn signup_page(session: Session) -> Result<HttpResponse> {
    if session_user_id(&session).is_some() {
        return Ok(redirect_after_login(&session));
    }

    render(SignupTemplate {
        error: String::new(),
        has_error: false,
    })
}

pub async fn signup(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<SignupForm>,
) -> Result<HttpResponse> {
    match services::register_customer(&data.db, form.into_inner()).await {
        Ok(user) => {
            session.insert("user_id", user.id)?;
            session.insert("role", user.role)?;
            Ok(redirect("/customer/dashboard"))
        }
        Err(error) => render(SignupTemplate {
            error,
            has_error: true,
        }),
    }
}

pub async fn logout(session: Session) -> Result<HttpResponse> {
    session.purge();
    Ok(redirect("/"))
}

fn redirect_after_login(session: &Session) -> HttpResponse {
    let role = session.get::<String>("role").ok().flatten();

    if role.as_deref() == Some("admin") {
        redirect("/admin/fixed-deposits")
    } else {
        redirect("/customer/dashboard")
    }
}
