#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Config
    let configuration =
        zero2prod::configuration::get("configuration").expect("Failed to read configuration");

    // telemetry
    zero2prod::telemetry::init_subscriber("zero2prod", "info", std::io::stdout);

    // Database
    let address = format!("0.0.0.0:{port}", port = configuration.application_port);
    let listener = std::net::TcpListener::bind(&address)?;
    let connection_string = configuration.database.connection_string();
    let pool =
        sqlx::PgPool::connect_lazy(&connection_string).expect("Failed to connect to Postgres");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    // Application
    zero2prod::startup::run(listener, pool)?.await
}
