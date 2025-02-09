#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Configuration
    let configuration = zero2prod::configuration::get().expect("Failed to read configuration");

    // Telemetry
    zero2prod::telemetry::init_subscriber("zero2prod", "info", std::io::stdout);
    // Application
    let application = zero2prod::startup::Application::build(&configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
