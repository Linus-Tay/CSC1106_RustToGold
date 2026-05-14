use crate::views::{render, HomeTemplate};
use actix_web::{HttpResponse, Result};

pub async fn home() -> Result<HttpResponse> {
    render(HomeTemplate)
}
