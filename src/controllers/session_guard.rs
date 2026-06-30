use crate::models::User;
use crate::repositories::user_repository;
use crate::AppState;
use actix_session::Session;
use actix_web::{http::header, web, HttpResponse, Result};
use uuid::Uuid;

pub async fn require_customer(
    data: &web::Data<AppState>,
    session: &Session,
) -> Result<User, HttpResponse> {
    let Some(user_id) = session_user_id(session) else {
        return Err(redirect("/login"));
    };

    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(e) => {
            println!("error here?");
            session.purge();
            return Err(redirect("/login"));
        }
    };

    println!("user_uuid: {}", user_uuid);

    let user = match user_repository::find_user_by_id(&data.db, user_uuid).await {
        Ok(user) => user,
        Err(e) => {
            println!("error from db: {}", e.to_string());
            session.purge();
            return Err(redirect("/login"))
        }
        _ => {
            session.purge();
            return Err(redirect("/login"));
        }
    };

    if !user.is_active() {
        session.purge();
        return Err(redirect("/login"));
    }

    if !user.is_customer() {
        return Err(redirect("/403"));
    }

    Ok(user)
}

pub fn session_user_id(session: &Session) -> Option<String> {
    session.get::<String>("user_id").ok().flatten()
}

pub fn redirect(path: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .append_header((header::LOCATION, path))
        .finish()
}
