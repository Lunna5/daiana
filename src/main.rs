use actix_web::{App, HttpServer};
use daiana::util::pretty_logger;
use log::{debug, info};



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_logger::init();

    info!("Hola");
    debug!("u.u");
    log::warn!("Warning");
    log::error!("Error");
    log::trace!("Trace");
    
    HttpServer::new(|| {
        App::new()
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}