use actix_web::{HttpResponse, Responder};

pub async fn health_check() -> impl Responder {
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
