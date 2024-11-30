use crate::ghost;
use crate::{
    config::Config,
    email::format_email,
    ghost::{fetch_subscribers, WebhookPayload},
};
use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{Request, StatusCode},
};
use hmac::{Hmac, Mac};
use resend_rs::{types::CreateEmailBaseOptions, Resend};
use sha2::Sha256;
use std::time::Duration;

// Define a reasonable size limit for webhook payloads (e.g., 5MB)
const MAX_BODY_SIZE: usize = 5 * 1024 * 1024;

// Constants for batch processing
const BATCH_SIZE: usize = 100; // Resend's max batch size
const BATCH_DELAY: Duration = Duration::from_secs(1);

pub async fn handle_webhook(
    State(config): State<Config>,
    request: Request<Body>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!("Received webhook request");

    // Extract and clone the signature header before consuming the body
    let signature_header = request
        .headers()
        .get("x-ghost-signature")
        .ok_or_else(|| {
            tracing::warn!("Missing x-ghost-signature header");
            StatusCode::UNAUTHORIZED
        })?
        .to_str()
        .map_err(|_| {
            tracing::error!("Invalid signature format in header");
            StatusCode::BAD_REQUEST
        })?
        .to_owned();

    tracing::debug!("Received signature header: {}", signature_header);

    // Now we can safely consume the request body
    tracing::debug!("Reading request body...");
    let body_bytes = to_bytes(request.into_body(), MAX_BODY_SIZE)
        .await
        .map_err(|e| {
            tracing::error!("Failed to read request body: {}", e);
            StatusCode::BAD_REQUEST
        })?;
    tracing::debug!("Received body of {} bytes", body_bytes.len());

    // Log the raw JSON for debugging
    let body_string = String::from_utf8_lossy(&body_bytes);
    tracing::debug!("Raw JSON body: {}", body_string);

    // Parse the body as JSON
    tracing::debug!("Parsing JSON body...");
    let payload: WebhookPayload = match serde_json::from_slice(&body_bytes) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to parse JSON body: {}", e);

            // Get line and column from the error
            let line = e.line();
            let column = e.column();

            tracing::error!("Error at line {} column {}", line, column);

            // Print the problematic part of the JSON
            let lines: Vec<&str> = body_string.lines().collect();
            if line > 0 && line <= lines.len() {
                tracing::error!("Problematic line: {}", lines[line - 1]);
                // Also print a few lines before and after for context
                let start = line.saturating_sub(3);
                let end = (line + 2).min(lines.len());
                for (i, line_content) in lines[start..end].iter().enumerate() {
                    tracing::error!("Line {}: {}", start + i + 1, line_content);
                }
            }

            return Err(StatusCode::BAD_REQUEST);
        }
    };

    tracing::info!(
        "Received webhook for post: {} (ID: {})",
        payload.post.current.title,
        payload.post.current.id
    );

    // Extract both signature and timestamp
    tracing::debug!("Parsing signature header parts...");
    let parts: Vec<&str> = signature_header.split(", ").collect();
    tracing::debug!("Signature header parts: {:?}", parts);

    let (signature, timestamp) = match (parts.first(), parts.get(1)) {
        (Some(sig), Some(ts)) => {
            let sig = sig.strip_prefix("sha256=").ok_or_else(|| {
                tracing::error!("Invalid signature format - missing sha256= prefix");
                StatusCode::BAD_REQUEST
            })?;
            let ts = ts.strip_prefix("t=").ok_or_else(|| {
                tracing::error!("Invalid timestamp format - missing t= prefix");
                StatusCode::BAD_REQUEST
            })?;
            tracing::debug!("Extracted signature: {}", sig);
            tracing::debug!("Extracted timestamp: {}", ts);
            (sig, ts)
        }
        _ => {
            tracing::error!("Invalid signature header format");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Try both old and new signature methods
    tracing::debug!("Verifying signature...");
    let body_string = String::from_utf8_lossy(&body_bytes);

    // Log the first and last few characters of the body and secret
    tracing::debug!(
        "Body preview: {}...{}",
        &body_string.chars().take(50).collect::<String>(),
        &body_string.chars().rev().take(50).collect::<String>()
    );
    tracing::debug!(
        "Secret preview: {}...{}",
        &config.webhook_secret.chars().take(10).collect::<String>(),
        &config
            .webhook_secret
            .chars()
            .rev()
            .take(10)
            .collect::<String>()
    );

    // Old method (pre 5.87.1) - just the body
    let mut mac =
        Hmac::<Sha256>::new_from_slice(config.webhook_secret.as_bytes()).map_err(|e| {
            tracing::error!("Failed to create HMAC: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    mac.update(body_string.as_bytes());
    let old_signature = hex::encode(mac.finalize().into_bytes());

    // New method (5.87.1+) - body + timestamp
    let mut mac =
        Hmac::<Sha256>::new_from_slice(config.webhook_secret.as_bytes()).map_err(|e| {
            tracing::error!("Failed to create HMAC: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let combined_string = format!("{}{}", body_string, timestamp);
    tracing::debug!("Combined string length: {}", combined_string.len());
    mac.update(combined_string.as_bytes());
    let new_signature = hex::encode(mac.finalize().into_bytes());

    tracing::debug!("Received signature: {}", signature);
    tracing::debug!("Old method signature: {}", old_signature);
    tracing::debug!("New method signature: {}", new_signature);

    // Check if either signature matches
    if signature != old_signature && signature != new_signature {
        tracing::warn!(
            "Invalid webhook signature. Expected old: {} or new: {}, Received: {}",
            old_signature,
            new_signature,
            signature
        );
        return Err(StatusCode::UNAUTHORIZED);
    }

    tracing::debug!("Webhook signature verified successfully");

    // Fetch settings once before processing emails
    let settings = ghost::fetch_settings(&config).await.map_err(|e| {
        tracing::error!("Failed to fetch Ghost settings: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Fetch subscribers
    let subscribers = fetch_subscribers(&config).await.map_err(|e| {
        tracing::error!("Failed to fetch subscribers: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Send emails using Resend in batches
    let resend_client = Resend::new(&config.resend_api_key);

    for (batch_index, subscriber_batch) in subscribers.chunks(BATCH_SIZE).enumerate() {
        tracing::info!(
            "Processing batch {} with {} subscribers",
            batch_index + 1,
            subscriber_batch.len()
        );

        let mut batch_emails = Vec::new();

        // Prepare all emails in this batch
        for subscriber in subscriber_batch {
            tracing::debug!("Preparing email for subscriber: {}", subscriber.email);

            let html_content = format_email(&payload.post.current, subscriber, &config, &settings)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to format email: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            let email = CreateEmailBaseOptions::new(
                &config.from_email,
                vec![subscriber.email.clone()],
                payload.post.current.title.clone(),
            )
            .with_html(&html_content);

            batch_emails.push(email);
        }

        // Send the entire batch using BatchSvc
        match resend_client.batch.send(batch_emails).await {
            Ok(responses) => {
                tracing::info!(
                    "Successfully sent batch {} ({} emails)",
                    batch_index + 1,
                    responses.len()
                );
                for response in responses {
                    tracing::debug!("Email sent with ID: {}", response.id);
                }
            }
            Err(e) => {
                tracing::error!("Failed to send batch {}: {}", batch_index + 1, e);
            }
        }

        // Sleep between batches to respect rate limits
        if batch_index < subscribers.chunks(BATCH_SIZE).len() - 1 {
            tracing::debug!("Sleeping for 1 second before next batch");
            tokio::time::sleep(BATCH_DELAY).await;
        }
    }

    tracing::info!("Webhook processing completed successfully");
    Ok(StatusCode::OK)
}
