use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use zero2prod::startup::run;
use zero2prod::telemetry::init_subscriber;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}
static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        init_subscriber("test", "info", std::io::stdout);
    } else {
        init_subscriber("test", "info", std::io::sink);
    }
});
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = std::net::TcpListener::bind("0.0.0.0:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://0.0.0.0:{port}");
    let configuration =
        zero2prod::configuration::get("configuration").expect("failed to fetch configuration");
    let connection_string = configuration.database.connection_string_without_db();
    let pool = configure_database(&connection_string).await;
    let server = run(listener, pool.clone()).expect("Failed to bind address");
    tokio::spawn(server);
    TestApp {
        address,
        db_pool: pool,
    }
}
pub async fn configure_database(connection_string: &str) -> PgPool {
    let mut connection = PgConnection::connect(connection_string)
        .await
        .expect("Failed to connect to Postgres");
    match connection.execute("CREATE DATABASE test_newsletters").await {
        Ok(_) => println!("Database for tests created"),
        Err(e) => println!("Failed to create database for tests: {e:?}"),
    }
    let connection_string = format!("{connection_string}/test_newsletters");
    let pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    pool
}

#[actix_web::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let uri = format!("{}/health_check", test_app.address);
    let response = client
        .get(&uri)
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_web::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let uri = format!("{}/subscriptions", test_app.address);
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&uri)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    sqlx::query!("DELETE FROM subscriptions WHERE email = $1", saved.email)
        .execute(&test_app.db_pool)
        .await
        .expect("Failed to execute request.");
}

#[actix_web::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let uri = format!("{}/subscriptions", test_app.address);
    let test_cases = [
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&uri)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message,
        );
    }
}
