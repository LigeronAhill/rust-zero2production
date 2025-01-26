use zero2prod::run;

fn spawn_app() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = run(port).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    port
}

#[actix_web::test]
async fn health_check_works() {
    let port = spawn_app();
    let client = reqwest::Client::new();
    let uri = format!("http://127.0.0.1:{port}/health_check");
    let response = client
        .get(&uri)
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
