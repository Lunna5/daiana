use actix_web::{get, Responder, HttpResponse, Scope};
use serde_json::json;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "ping": "pong",
        "version": VERSION,
    }))
}

pub fn endpoints(scope: Scope) -> Scope {
    scope.service(index)
}