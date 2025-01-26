use zero2prod::startup::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configuration =
        zero2prod::configuration::get("configuration").expect("Failed to read configuration");
    let address = format!("0.0.0.0:{}", configuration.application_port);
    let listener = std::net::TcpListener::bind(&address)?;
    let connection_string = configuration.database.connection_string();
    let pool = sqlx::PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");
    run(listener, pool)?.await
}
