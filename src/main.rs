use actix_web::middleware::{Compress, Logger, NormalizePath, TrailingSlash};
use actix_web::web::Data;
use actix_web::{rt, web, App, HttpServer};
use daiana::channel::ChannelManager;
use daiana::util::pretty_logger;
use daiana::{AppState, service};
use log::{debug, info};
use std::env;
use actix_web::rt::time::sleep;
use std::time::Duration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    pretty_logger::init();

    let host: String = env::var("HOST").unwrap_or(String::from("0.0.0.0"));
    let port: String = env::var("PORT").unwrap_or(String::from("2022"));

    info!("Initializing daiana...");
    debug!("Debug mode enabled");

    let app_state = Data::new(AppState {
        channel_manager: ChannelManager::new(),
    });

    let gc_state = app_state.clone();

    let gc_interval = env::var("CHANNEL_TIMEOUT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);

    rt::spawn(async move {
        loop {
            sleep(Duration::from_secs(gc_interval)).await;
            gc_state.channel_manager.clean_empty_channels(gc_interval);
        }
    });

    HttpServer::new(move || {
        let mut app = App::new()
            .wrap(Compress::default())
            .wrap(NormalizePath::new(TrailingSlash::MergeOnly))
            .wrap(Logger::default());

        app = app.app_data(app_state.clone());

        app = app.service(service::room::endpoints(web::scope("/room")));
        app = app.service(service::health::endpoints(web::scope("")));
        app
    })
    .bind(host + ":" + &*port)?
    .run()
    .await
}
