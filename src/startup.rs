use crate::configuration::{DatabaseSettings, Settings};
use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}
impl Application {
    pub async fn build(configuration: &Settings) -> Result<Self, std::io::Error> {
        // Database
        let address = format!("0.0.0.0:{port}", port = configuration.application.port);
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr()?.port();
        let pool = get_connection_pool(&configuration.database);
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("Failed to migrate database");

        // Email Client
        let base_url = &configuration.email.base_url;
        let sender =
            SubscriberEmail::parse(&configuration.email.sender).expect("Failed to parse sender");
        let api_key = &configuration.email.apikey;
        let timeout = configuration.email.timeout();
        let email_client = EmailClient::new(base_url, sender, api_key, timeout)
            .expect("Failed to create email client");
        let server = run(listener, pool, email_client)?;
        Ok(Self { port, server })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

fn run(
    tcp_listener: TcpListener,
    pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let pool = web::Data::new(pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(pool.clone())
            .app_data(email_client.clone())
    })
    .listen(tcp_listener)?
    .run();
    Ok(server)
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}
