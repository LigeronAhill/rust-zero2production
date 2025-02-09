use crate::domain::SubscriberEmail;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
pub struct EmailClient {
    base_url: String,
    client: Client,
    sender: SubscriberEmail,
    api_key: String,
}

impl EmailClient {
    pub fn new(
        base_url: &str,
        sender: SubscriberEmail,
        api_key: &str,
        timeout: std::time::Duration,
    ) -> Result<Self, String> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            base_url: base_url.to_owned(),
            client,
            sender,
            api_key: api_key.to_owned(),
        })
    }
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        body: &str,
    ) -> Result<(), String> {
        let uri = format!("{}/ru/api/sendEmail", self.base_url);
        let params = RequestParams::builder(&self.api_key)
            .sender_email(self.sender.as_ref())
            .email(recipient.as_ref())
            .subject(subject)
            .body(body);
        let response = self
            .client
            .get(&uri)
            .query(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        let body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        match serde_json::from_value::<EmailSendResponse>(body.clone()) {
            Ok(result) => {
                let item = result.result;
                tracing::info!("BODY: \n{body:?}");
                tracing::info!(
                    "index: {index:?}; email_id: {email_id:?}",
                    index = item.index,
                    email_id = item.email_id
                );
                if let Some(errors) = item.errors {
                    for err in errors {
                        tracing::error!(
                            "code: {code:?}; message: {message:?}",
                            code = err.code,
                            message = err.message
                        );
                    }
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "error sending email");
                tracing::error!("{body:#?}");
                return Err(e.to_string());
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Default)]
struct RequestParams {
    format: String,
    api_key: String,
    email: String,
    sender_name: String,
    sender_email: String,
    subject: String,
    body: String,
    list_id: String,
}
impl RequestParams {
    fn builder(api_key: &str) -> Self {
        let mut params = HashMap::new();
        params.insert("api_key".to_owned(), api_key.to_owned());
        params.insert("format".to_owned(), "json".to_owned());
        params.insert("list_id".to_owned(), "1".to_owned());
        params.insert("sender_name".to_owned(), "zero2prod".to_owned());
        RequestParams {
            format: "json".to_string(),
            api_key: api_key.to_owned(),
            list_id: "1".to_string(),
            sender_name: "zero2prod".to_string(),
            ..Self::default()
        }
    }
    fn sender_email(mut self, sender_email: &str) -> Self {
        self.sender_email = sender_email.to_string();
        self
    }
    fn email(mut self, email: &str) -> Self {
        self.email = email.to_string();
        self
    }
    fn subject(mut self, subject: &str) -> Self {
        self.subject = subject.to_string();
        self
    }
    fn body(mut self, body: &str) -> Self {
        self.body = body.to_string();
        self
    }
}

#[derive(Deserialize, Debug, Clone)]
struct EmailSendResponse {
    result: EmailSendResult,
}

#[derive(Deserialize, Debug, Clone)]
struct EmailSendResult {
    index: Option<i64>,
    email_id: String,
    errors: Option<Vec<EmailSendError>>,
}

#[derive(Deserialize, Debug, Clone)]
struct EmailSendError {
    code: String,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use serde_json::json;
    use wiremock::matchers::{any, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Generate a random email subject
    fn subject() -> String {
        Sentence(1..2).fake()
    }
    /// Generate a random email content
    fn content() -> String {
        Paragraph(1..10).fake()
    }
    /// Generate a random subscriber email
    fn email() -> SubscriberEmail {
        let addr: String = SafeEmail().fake();
        SubscriberEmail::parse(&addr).unwrap()
    }
    /// Get a test instance of`EmailClient`
    fn email_client(base_url: String) -> EmailClient {
        let api_key: String = Faker.fake();
        let timeout = std::time::Duration::from_millis(200);
        EmailClient::new(&base_url, email(), &api_key, timeout).unwrap()
    }
    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url_and_succeed_when_200() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        let subscriber_email = email();
        let subject = subject();
        let content = content();
        let body = json!({
            "result": {
              "email_id": "some id"
            }
        });
        Mock::given(path("/ru/api/sendEmail"))
            .and(method("GET"))
            .and(query_param("format", "json"))
            .and(query_param("email", subscriber_email.as_ref()))
            .and(query_param("list_id", "1"))
            .and(query_param("subject", &subject))
            .and(query_param("body", &content))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .expect(1)
            .mount(&mock_server)
            .await;
        // Act
        let result = email_client
            .send_email(subscriber_email, &subject, &content)
            .await;
        // Assert
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        let subscriber_email = email();
        let subject = subject();
        let content = content();
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;
        // Act
        let outcome = email_client
            .send_email(subscriber_email, &subject, &content)
            .await;
        // Assert
        assert!(outcome.is_err());
    }
    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        let subscriber_email = email();
        let subject = subject();
        let content = content();
        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;
        // Act
        let outcome = email_client
            .send_email(subscriber_email, &subject, &content)
            .await;
        // Assert
        assert!(outcome.is_err());
    }
}
