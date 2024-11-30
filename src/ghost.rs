use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize, Serialize)]
pub struct WebhookPayload {
    pub post: PostWrapper,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PostWrapper {
    pub current: Post,
    pub previous: PreviousPost,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Post {
    pub id: String,
    pub uuid: String,
    pub title: String,
    pub slug: String,
    pub html: String,
    pub comment_id: String,
    pub plaintext: String,
    pub feature_image: Option<String>,
    pub featured: bool,
    pub status: String,
    pub visibility: String,
    pub created_at: String,
    pub updated_at: String,
    pub published_at: String,
    pub url: String,
    pub excerpt: String,
    pub primary_author: Author,
    pub reading_time: u32,
    pub feature_image_alt: Option<String>,
    pub feature_image_caption: Option<String>,
    #[serde(flatten)]
    pub other: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PreviousPost {
    pub status: String,
    pub updated_at: String,
    pub published_at: Option<String>,
    #[serde(flatten)]
    pub other: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Author {
    pub name: String,
    pub profile_image: Option<String>,
    pub bio: Option<String>,
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MembersResponse {
    pub members: Vec<Member>,
    pub meta: Meta,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Member {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(flatten)]
    pub other: Value,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Meta {
    pub pagination: Pagination,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Pagination {
    pub page: u32,
    pub limit: u32,
    pub pages: u32,
    pub total: u32,
    pub next: Option<u32>,
    pub prev: Option<u32>,
}

#[derive(Debug, Serialize)]
struct Claims {
    aud: String,
    exp: u64,
    iat: u64,
}

// Constants for batch processing
const BATCH_SIZE: usize = 100; // Resend's max batch size

pub async fn fetch_subscribers(config: &crate::config::Config) -> anyhow::Result<Vec<Member>> {
    let client = reqwest::Client::new();
    let mut all_members = Vec::new();
    let mut current_page = 1;

    loop {
        let url = format!(
            "{}/ghost/api/admin/members/?page={}&limit={}",
            config.ghost_url,
            current_page,
            BATCH_SIZE // Use same batch size for consistency
        );

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let claims = Claims {
            aud: "/admin/".to_string(),
            exp: now + 300, // 5 minutes
            iat: now,
        };

        // Create header with the ID
        let mut header = Header::new(jsonwebtoken::Algorithm::HS256);
        header.kid = Some(config.ghost_admin_id.clone());
        header.typ = Some("JWT".to_string());

        // Decode the hex secret into bytes
        let secret_bytes = hex::decode(&config.ghost_admin_secret).map_err(|e| {
            tracing::error!("Failed to decode hex secret: {}", e);
            anyhow::anyhow!("Invalid hex secret")
        })?;

        tracing::debug!("Creating JWT with ID: {}", config.ghost_admin_id);

        let token = encode(&header, &claims, &EncodingKey::from_secret(&secret_bytes))?;

        tracing::debug!("Generated token: {}", token);

        let response = client
            .get(&url)
            .header("Authorization", format!("Ghost {}", token))
            .header("Accept-Version", "v5.0")
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to send request to Ghost API: {}", e);
                e
            })?;

        let status = response.status();
        tracing::debug!(
            "Ghost API response status for page {}: {}",
            current_page,
            status
        );

        if !status.is_success() {
            let error_body = response.text().await?;
            tracing::error!("Ghost API error response: {}", error_body);
            return Err(anyhow::anyhow!("Ghost API returned error: {}", status));
        }

        let body = response.text().await?;
        let response: MembersResponse = serde_json::from_str(&body)?;

        tracing::debug!(
            "Fetched {} members from page {} of {}",
            response.members.len(),
            current_page,
            response.meta.pagination.pages
        );

        // Add members from this page to our collection
        all_members.extend(response.members);

        // Check if we've reached the last page
        if current_page >= response.meta.pagination.pages {
            break;
        }

        current_page += 1;

        // Optional: sleep between pages to be nice to the Ghost API
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    tracing::debug!("Successfully fetched {} total members", all_members.len());
    Ok(all_members)
}
