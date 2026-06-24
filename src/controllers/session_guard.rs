use crate::models::User;
use crate::repositories::user_repository;
use crate::AppState;
use actix_session::Session;
use actix_web::{http::header, web, HttpResponse, Result};

pub async fn require_customer(
    data: &web::Data<AppState>,
    session: &Session,
) -> Result<User, HttpResponse> {
    let Some(user_id) = session_user_id(session) else {
        return Err(redirect("/login"));
    };

    let user = match user_repository::find_user_by_id(&data.db, user_id).await {
        Ok(Some(user)) => user,
        _ => {
            session.purge();
            return Err(redirect("/login"));
        }
    };

    if !user.is_active() {
        session.purge();
        return Err(redirect("/login"));
    }

    /*if !user.is_customer() {
        return Err(redirect("/403"));
    }*/

    Ok(user)
}

pub async fn require_staff_or_admin(
    data: &web::Data<AppState>,
    session: &Session,
) -> Result<User, HttpResponse> {
    let Some(user_id) = session_user_id(session) else {
        return Err(redirect("/login"));
    };

    let user = match user_repository::find_user_by_id(&data.db, user_id).await {
        Ok(Some(user)) => user,
        _ => {
            session.purge();
            return Err(redirect("/login"));
        }
    };

    if !user.is_active() {
        session.purge();
        return Err(redirect("/login"));
    }

    if user.role != "admin" && user.role != "staff" {
        return Err(redirect("/403"));
    }

    Ok(user)
}

pub async fn require_admin(
    data: &web::Data<AppState>,
    session: &Session,
) -> Result<User, HttpResponse> {
    let Some(user_id) = session_user_id(session) else {
        return Err(redirect("/login"));
    };

    let user = match user_repository::find_user_by_id(&data.db, user_id).await {
        Ok(Some(user)) => user,
        _ => {
            session.purge();
            return Err(redirect("/login"));
        }
    };

    if !user.is_active() {
        session.purge();
        return Err(redirect("/login"));
    }

    if user.role != "admin" {
        return Err(redirect("/403"));
    }

    Ok(user)
}

pub fn session_user_id(session: &Session) -> Option<i64> {
    session.get::<i64>("user_id").ok().flatten()
}

pub fn redirect(path: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .append_header((header::LOCATION, path))
        .finish()
}
