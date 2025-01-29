#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Config
    let configuration = zero2prod::configuration::get().expect("Failed to read configuration");

    // telemetry
    zero2prod::telemetry::init_subscriber("zero2prod", "info", std::io::stdout);

    // Database
    let address = format!("0.0.0.0:{port}", port = configuration.application.port);
    let listener = std::net::TcpListener::bind(&address)?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_lazy_with(configuration.database.with_db());
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    // Application
    zero2prod::startup::run(listener, pool)?.await
}
