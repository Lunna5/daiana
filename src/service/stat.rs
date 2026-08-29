use crate::AppState;
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder, get};
use serde_json::json;

#[get("/")]
async fn index(data: Data<AppState>) -> impl Responder {
    let channel_manager = &data.channel_manager;
    let mut active_rooms: u32 = 0;
    let mut active_clients: u32 = 0;

    {
        channel_manager
            .channels
            .lock()
            .expect("Unable to lock channel")
            .iter()
            .for_each(|(_, channel)| {
                if !channel.clients.is_empty() {
                    active_rooms += 1;
                    active_clients += channel.clients.len() as u32;
                }
            });
    }

    HttpResponse::Ok().json(json!({
        "active_rooms": active_rooms,
        "active_clients": active_clients,
    }))
}

pub fn endpoints(scope: actix_web::Scope) -> actix_web::Scope {
    scope.service(index)
}
