use once_cell::sync::Lazy;
use reqwest::Response;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use wiremock::MockServer;
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::telemetry::init_subscriber;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}
impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> Result<Response, reqwest::Error> {
        reqwest::Client::new()
            .post(format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
    }
}
static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        init_subscriber("test", "info", std::io::stdout);
    } else {
        init_subscriber("test", "info", std::io::sink);
    }
});
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let email_server = MockServer::start().await;
    let mut configuration = zero2prod::configuration::get().expect("failed to fetch configuration");
    configuration.application.port = 0;
    configuration.database.database_name = String::from("test_newsletters");
    configuration.email.base_url = email_server.uri();
    configure_database(&configuration.database).await;
    let application = Application::build(&configuration)
        .await
        .expect("failed to build application");
    let address = format!("http://127.0.0.1:{port}", port = application.port());
    tokio::spawn(application.run_until_stopped());
    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
    }
}
pub async fn configure_database(config: &zero2prod::configuration::DatabaseSettings) {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    match connection.execute("CREATE DATABASE test_newsletters").await {
        Ok(_) => tracing::info!("Database for tests created"),
        Err(e) => tracing::error!("Failed to create database for tests: {e:?}"),
    }
    let pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
}
