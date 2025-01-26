use std::net::TcpListener;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};

pub fn run(port: u16) -> Result<actix_web::dev::Server, std::io::Error> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))?;
    let server =
        HttpServer::new(move || App::new().route("/health_check", web::get().to(health_check)))
            .listen(listener)?
            .run();
    Ok(server)
}
async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_health_check() {
        let app =
            test::init_service(App::new().route("/health_check", web::get().to(health_check)))
                .await;
        let req = test::TestRequest::get().uri("/health_check").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
