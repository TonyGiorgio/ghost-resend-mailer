use crate::ghost::{Member, Post};
use serde::Serialize;

#[derive(Serialize)]
struct EmailTemplate {
    site: SiteInfo,
    post: PostContent,
    newsletter: NewsletterInfo,
}

#[derive(Serialize)]
struct SiteInfo {
    url: String,
    title: String,
    description: String,
    color: String,
}

#[derive(Serialize)]
struct PostContent {
    id: String,
    url: String,
    title: String,
    html: String,
    excerpt: String,
    author: String,
    author_image: Option<String>,
    author_bio: Option<String>,
    author_url: String,
    feature_image: Option<String>,
    feature_image_alt: Option<String>,
    feature_image_caption: Option<String>,
    reading_time: u32,
}

#[derive(Serialize)]
struct NewsletterInfo {
    subscription_link: String,
    unsubscribe_link: String,
}

pub async fn format_email(
    post: &Post,
    member: &Member,
    config: &crate::config::Config,
) -> anyhow::Result<String> {
    let template = EmailTemplate {
        site: SiteInfo {
            url: config.ghost_url.clone(),
            title: "Your Blog".to_string(),
            description: "Your blog description".to_string(),
            color: "#ff247c".to_string(),
        },
        post: PostContent {
            id: post.id.clone(),
            url: post.url.clone(),
            title: post.title.clone(),
            html: post.html.clone(),
            excerpt: post.excerpt.clone(),
            author: post.primary_author.name.clone(),
            author_image: post.primary_author.profile_image.clone(),
            author_bio: post.primary_author.bio.clone(),
            author_url: post.primary_author.url.clone(),
            feature_image: post.feature_image.clone(),
            feature_image_alt: post.feature_image_alt.clone(),
            feature_image_caption: post.feature_image_caption.clone(),
            reading_time: post.reading_time,
        },
        newsletter: NewsletterInfo {
            subscription_link: format!("{}#/portal/account", config.ghost_url),
            unsubscribe_link: format!(
                "{}#/portal/account?action=unsubscribe&uuid={}",
                config.ghost_url, member.id
            ),
        },
    };

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width">
            <style>
                body {{
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
                    line-height: 1.6;
                    color: #333;
                    max-width: 600px;
                    margin: 0 auto;
                    padding: 20px;
                }}
                .header {{ text-align: center; margin-bottom: 30px; }}
                .post-title {{ font-size: 24px; font-weight: bold; }}
                .post-excerpt {{ font-style: italic; color: #666; }}
                .post-content {{ margin: 20px 0; }}
                .footer {{ text-align: center; font-size: 14px; color: #666; margin-top: 30px; }}
                .button {{
                    display: inline-block;
                    padding: 10px 20px;
                    background-color: {site_color};
                    color: white;
                    text-decoration: none;
                    border-radius: 5px;
                }}
                .author-info {{ 
                    display: flex; 
                    align-items: center;
                    margin: 20px 0;
                }}
                .author-image {{
                    width: 50px;
                    height: 50px;
                    border-radius: 25px;
                    margin-right: 15px;
                }}
                .reading-time {{
                    color: #666;
                    font-size: 14px;
                    margin-bottom: 20px;
                }}
                .feature-image {{
                    max-width: 100%;
                    height: auto;
                    margin: 20px 0;
                }}
                .feature-caption {{
                    font-size: 14px;
                    color: #666;
                    text-align: center;
                }}
            </style>
        </head>
        <body>
            <div class="header">
                <h1>{site_title}</h1>
            </div>

            <div class="post-title">
                <h1>{post_title}</h1>
                <div class="reading-time">
                    {reading_time} min read
                </div>
            </div>

            {feature_image_html}

            <div class="author-info">
                {author_image_html}
                <div>
                    <strong>{author_name}</strong>
                    {author_bio_html}
                </div>
            </div>

            <div class="post-excerpt">
                {post_excerpt}
            </div>

            <div class="post-content">
                {post_content}
            </div>

            <div class="footer">
                <p>
                    <a href="{subscription_link}">Manage subscription</a> | 
                    <a href="{unsubscribe_link}">Unsubscribe</a>
                </p>
            </div>
        </body>
        </html>
    "#,
        site_color = template.site.color,
        site_title = template.site.title,
        post_title = template.post.title,
        reading_time = template.post.reading_time,
        feature_image_html = template
            .post
            .feature_image
            .as_ref()
            .map_or(String::new(), |url| {
                format!(
                    r#"<img src="{}" alt="{}" class="feature-image">{}"#,
                    url,
                    template.post.feature_image_alt.as_deref().unwrap_or(""),
                    template
                        .post
                        .feature_image_caption
                        .as_ref()
                        .map_or(String::new(), |caption| {
                            format!(r#"<div class="feature-caption">{}</div>"#, caption)
                        })
                )
            }),
        author_image_html = template
            .post
            .author_image
            .as_ref()
            .map_or(String::new(), |url| {
                format!(
                    r#"<img src="{}" alt="{}" class="author-image">"#,
                    url, template.post.author
                )
            }),
        author_name = template.post.author,
        author_bio_html = template
            .post
            .author_bio
            .as_ref()
            .map_or(String::new(), |bio| {
                format!(r#"<div class="author-bio">{}</div>"#, bio)
            }),
        post_excerpt = template.post.excerpt,
        post_content = template.post.html,
        subscription_link = template.newsletter.subscription_link,
        unsubscribe_link = template.newsletter.unsubscribe_link
    );

    Ok(html)
}
