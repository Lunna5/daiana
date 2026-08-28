//! Health check HTTP endpoint (`GET /`).

use actix_web::{HttpResponse, Responder, Scope, get};
use serde_json::json;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Responds with ping-pong health status and current package version.
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "ping": "pong",
        "version": VERSION,
    }))
}

/// Registers the health check routes into the provided Actix [`Scope`].
pub fn endpoints(scope: Scope) -> Scope {
    scope.service(index)
}
