use zero2prod::startup::Application;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Config
    let configuration = zero2prod::configuration::get().expect("Failed to read configuration");

    // telemetry
    zero2prod::telemetry::init_subscriber("zero2prod", "info", std::io::stdout);

    // Application
    let application = Application::build(&configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
