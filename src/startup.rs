use crate::routes::{health_check, subscribe};
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(
    tcp_listener: TcpListener,
    pool: sqlx::PgPool,
) -> Result<actix_web::dev::Server, std::io::Error> {
    let pool = web::Data::new(pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(pool.clone())
    })
    .listen(tcp_listener)?
    .run();
    Ok(server)
}
