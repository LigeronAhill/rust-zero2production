use crate::domain::SubscriberEmail;
use reqwest::Client;
use serde::Serialize;

pub struct EmailClient {
    base_url: String,
    client: Client,
    sender: SubscriberEmail,
    api_key: String,
}

impl EmailClient {
    pub fn new(
        base_url: &str,
        sender: &str,
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
            sender: SubscriberEmail::parse(sender)?,
            api_key: api_key.to_owned(),
        })
    }
    pub async fn send_email(
        &self,
        recipient: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), String> {
        let uri = format!("{}/ru/api/sendEmail", self.base_url);
        let params = RequestParams::builder(&self.api_key)
            .sender_email(self.sender.as_ref())
            .email(recipient)
            .subject(subject)
            .body(body);
        let _response = self
            .client
            .get(&uri)
            .query(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        Ok(())
        // if response.status().is_success() {
        //     let body: EmailSendResponse = response.json().await.map_err(|e| e.to_string())?;
        //     for item in body.result {
        //         tracing::debug!(
        //             "index: {index}; email: {email}",
        //             index = item.index,
        //             email = item.email
        //         );
        //         for err in item.errors {
        //             tracing::error!(
        //                 "code: {code}; message: {message}",
        //                 code = err.code,
        //                 message = err.message
        //             );
        //         }
        //     }
        //     Ok(())
        // } else {
        //     Err(response.text().await.map_err(|e| e.to_string())?)
        // }
    }
}

#[derive(Serialize, Default)]
struct RequestParams<'a> {
    format: &'a str,
    api_key: &'a str,
    email: &'a str,
    sender_name: &'a str,
    sender_email: &'a str,
    subject: &'a str,
    body: &'a str,
    list_id: &'a str,
}
impl<'a> RequestParams<'a> {
    fn builder(api_key: &'a str) -> Self {
        RequestParams {
            format: "json",
            api_key,
            list_id: "1",
            sender_name: "zero2prod",
            ..Self::default()
        }
    }
    fn sender_email(mut self, sender_email: &'a str) -> Self {
        self.sender_email = sender_email;
        self
    }
    fn email(mut self, email: &'a str) -> Self {
        self.email = email;
        self
    }
    fn subject(mut self, subject: &'a str) -> Self {
        self.subject = subject;
        self
    }
    fn body(mut self, body: &'a str) -> Self {
        self.body = body;
        self
    }
}

// #[derive(Deserialize, Debug, Clone)]
// struct EmailSendResponse {
//     result: Vec<EmailSendResult>,
// }
//
// #[derive(Deserialize, Debug, Clone)]
// struct EmailSendResult {
//     index: i64,
//     email: String,
//     errors: Vec<EmailSendError>,
// }
//
// #[derive(Deserialize, Debug, Clone)]
// struct EmailSendError {
//     code: String,
//     message: String,
// }
//
#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
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
    fn email() -> String {
        SafeEmail().fake()
    }
    /// Get a test instance of`EmailClient`
    fn email_client(base_url: String) -> EmailClient {
        let api_key: String = Faker.fake();
        let timeout = std::time::Duration::from_millis(200);
        EmailClient::new(&base_url, &email(), &api_key, timeout).unwrap()
    }
    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url_and_succeed_when_200() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        let subscriber_email = email();
        let subject = subject();
        let content = content();
        Mock::given(path("/ru/api/sendEmail"))
            .and(method("GET"))
            .and(query_param("format", "json"))
            .and(query_param("email", &subscriber_email))
            .and(query_param("list_id", "1"))
            .and(query_param("subject", &subject))
            .and(query_param("body", &content))
            .respond_with(ResponseTemplate::new(200))
            .expect(1..3)
            .mount(&mock_server)
            .await;
        // Act
        let result = email_client
            .send_email(&subscriber_email, &subject, &content)
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
            .send_email(&subscriber_email, &subject, &content)
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
            .send_email(&subscriber_email, &subject, &content)
            .await;
        // Assert
        assert!(outcome.is_err());
    }
}
