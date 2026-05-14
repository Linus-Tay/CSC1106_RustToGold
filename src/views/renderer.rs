use actix_web::{error::ErrorInternalServerError, HttpResponse, Result};
use askama::Template;

pub fn render<T: Template>(template: T) -> Result<HttpResponse> {
    let html = template.render().map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
