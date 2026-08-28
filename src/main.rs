use std::env;
use std::sync::{Arc, Mutex};
use actix_web::{App, HttpServer, web};
use actix_web::middleware::{Compress, Logger, NormalizePath, TrailingSlash};
use actix_web::web::Data;
use daiana::util::pretty_logger;
use log::{debug, info};
use daiana::{service, AppState};
use daiana::channel::ChannelManager;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    pretty_logger::init();

    let host: String = env::var("HOST").unwrap_or(String::from("0.0.0.0"));
    let port: String = env::var("PORT").unwrap_or(String::from("2022"));

    info!("Initializing daiana...");
    debug!("Debug mode enabled");

    let app_state = Data::new(AppState {
        channel_manager: Arc::new(Mutex::new(ChannelManager::new())),
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