use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
subscriber_email = %form.email,
subscriber_name= %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    match SubscriberName::parse(form.0.name) {
        Ok(name) => match SubscriberEmail::parse(form.0.email) {
            Ok(email) => {
                let new_subscriber = NewSubscriber { name, email };
                match insert_subscriber(&pool, &new_subscriber).await {
                    Ok(_) => HttpResponse::Ok().finish(),
                    Err(_) => HttpResponse::InternalServerError().finish(),
                }
            }
            Err(_) => HttpResponse::BadRequest().finish(),
        },
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, subscriber)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
INSERT INTO subscriptions (id, email, name, subscribed_at)
VALUES ($1, $2, $3, $4)
"#,
        Uuid::new_v4(),
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {e:?}");
        e
    })?;
    Ok(())
}
