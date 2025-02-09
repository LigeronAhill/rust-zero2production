use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::spawn_app;

// #[actix_web::test]
// async fn subscribe_returns_a_200_for_valid_form_data() {
//     let test_app = spawn_app().await;
//     let body = json!(
//         {
//             "result": {
//                 "email_id": "some id"
//             }
//         }
//     );
//     Mock::given(path("/ru/api/sendEmail"))
//         .and(method("GET"))
//         .respond_with(ResponseTemplate::new(200).set_body_json(body))
//         .mount(&test_app.email_server)
//         .await;
//     let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
//     let response = test_app.post_subscriptions(body.into()).await.unwrap();
//     assert_eq!(200, response.status().as_u16());
//     let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
//         .fetch_one(&test_app.db_pool)
//         .await
//         .expect("Failed to fetch saved subscription.");
//     assert_eq!(saved.email, "ursula_le_guin@gmail.com");
//     assert_eq!(saved.name, "le guin");
//     sqlx::query!("DELETE FROM subscriptions WHERE email = $1", saved.email)
//         .execute(&test_app.db_pool)
//         .await
//         .expect("Failed to execute request.");
// }

#[actix_web::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let test_app = spawn_app().await;
    let test_cases = [
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = test_app
            .post_subscriptions(invalid_body.into())
            .await
            .unwrap();
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message,
        );
    }
}

#[actix_web::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_empty() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        // Act
        let response = app.post_subscriptions(body.into()).await.unwrap();
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}

#[actix_web::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/ru/api/sendEmail"))
        .and(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;
    // Act
    app.post_subscriptions(body.into()).await.unwrap();
    // Assert
    // Mock asserts on drop
    sqlx::query("DELETE FROM subscriptions WHERE email = $1")
        .bind("ursula_le_guin@gmail.com")
        .execute(&app.db_pool)
        .await
        .unwrap();
}

// #[tokio::test]
// async fn subscribe_sends_a_confirmation_email_with_a_link() {
//     // Arrange
//     let app = spawn_app().await;
//     let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
//     let resp_body = json!(
//         {
//             "result": {
//                 "email_id": "some id"
//             }
//         }
//     );
//     Mock::given(path("/ru/api/sendEmail"))
//         .and(method("GET"))
//         .respond_with(ResponseTemplate::new(200).set_body_json(resp_body))
//         .expect(1)
//         .mount(&app.email_server)
//         .await;
//     // Act
//     app.post_subscriptions(body.into()).await.unwrap();
//     // Assert
//     // Get the first intercepted request
//     let email_request = app.email_server.received_requests().await.unwrap();
//     assert!(!email_request.is_empty());
//     // Parse the body as JSON, starting from raw bytes
//     match serde_json::from_slice::<serde_json::Value>(&email_request.first().unwrap().body) {
//         Ok(body) => {
//             let get_link = |s: &str| {
//                 let links: Vec<_> = linkify::LinkFinder::new()
//                     .links(s)
//                     .filter(|l| *l.kind() == linkify::LinkKind::Url)
//                     .collect();
//                 assert_eq!(links.len(), 1);
//                 links[0].as_str().to_owned()
//             };
//             let html_link = get_link(body["HtmlBody"].as_str().unwrap());
//             let text_link = get_link(body["TextBody"].as_str().unwrap());
//             // The two links should be identical
//             assert_eq!(html_link, text_link);
//         }
//         Err(e) => {
//             assert_eq!(e.to_string(), String::new());
//         }
//     }
// }
