use actix_web::{test, web, App};
use daiana::{service, AppState};
use daiana::channel::ChannelManager;
use serde_json::Value;
use uuid::Uuid;

#[actix_web::test]
async fn test_health_endpoint() {
    let app = test::init_service(
        App::new().service(service::health::endpoints(web::scope("")))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["ping"], "pong");
    assert!(body["version"].is_string());
}

#[actix_web::test]
async fn test_create_room_endpoint() {
    let app_state = web::Data::new(AppState {
        channel_manager: ChannelManager::new(),
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .service(service::room::endpoints(web::scope("/room")))
    ).await;

    let req = test::TestRequest::post().uri("/room/").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    let room_id_str = body["id"].as_str().expect("Room ID should be a string");
    let room_uuid = Uuid::parse_str(room_id_str).expect("Room ID should be a valid UUID");

    assert!(app_state.channel_manager.channel_exists(room_uuid));
}

#[actix_web::test]
async fn test_connect_to_invalid_room_uuid() {
    let app_state = web::Data::new(AppState {
        channel_manager: ChannelManager::new(),
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .service(service::room::endpoints(web::scope("/room")))
    ).await;

    let req = test::TestRequest::get().uri("/room/invalid-uuid").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_connect_to_nonexistent_room() {
    let app_state = web::Data::new(AppState {
        channel_manager: ChannelManager::new(),
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .service(service::room::endpoints(web::scope("/room")))
    ).await;

    let random_uuid = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/room/{}", random_uuid))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
}
