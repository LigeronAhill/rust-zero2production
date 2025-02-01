use crate::helpers::spawn_app;

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
