use crate::ghost;
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
    settings: &ghost::Settings,
) -> anyhow::Result<String> {
    let template = EmailTemplate {
        site: SiteInfo {
            url: settings.url.clone(),
            title: settings.title.clone(),
            description: settings.description.clone(),
            color: settings
                .accent_color
                .clone()
                .unwrap_or_else(|| "#3eb0ef".to_string()),
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
        <html lang="en">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <style>
                /* Reset styles */
                body, div, p, h1, h2 {{
                    margin: 0;
                    padding: 0;
                }}

                /* Base styles */
                body {{
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
                    line-height: 1.6;
                    color: #333;
                    background: #ffffff;
                    padding: 0;
                    margin: 0;
                }}

                /* Container */
                .container {{
                    max-width: 600px;
                    margin: 0 auto;
                    padding: 40px 20px;
                }}

                /* Header */
                .header {{
                    text-align: center;
                    padding-bottom: 30px;
                    border-bottom: 1px solid #e5eff5;
                    margin-bottom: 30px;
                }}

                .header h1 {{
                    font-size: 28px;
                    font-weight: 600;
                    margin-bottom: 10px;
                }}

                /* Post title */
                .post-title {{
                    font-size: 32px;
                    line-height: 1.3;
                    font-weight: 700;
                    margin-bottom: 20px;
                }}

                /* Feature image */
                .feature-image {{
                    width: 100%;
                    height: auto;
                    margin: 30px 0;
                    border-radius: 5px;
                }}

                .feature-caption {{
                    font-size: 14px;
                    color: #738a94;
                    text-align: center;
                    margin-top: 10px;
                }}

                /* Author info */
                .author-info {{
                    display: flex;
                    align-items: center;
                    margin: 30px 0;
                }}

                .author-image {{
                    width: 60px;
                    height: 60px;
                    border-radius: 100%;
                    margin-right: 15px;
                }}

                .author-name {{
                    font-weight: 600;
                    font-size: 16px;
                }}

                .author-bio {{
                    color: #738a94;
                    font-size: 14px;
                    margin-top: 5px;
                }}

                /* Content */
                .content {{
                    font-size: 16px;
                    line-height: 1.7;
                    margin: 0 auto;
                }}

                .content p {{
                    margin-bottom: 1.5em;
                }}

                .content img {{
                    max-width: 100%;
                    height: auto;
                    margin: 30px 0;
                }}

                /* Footer */
                .footer {{
                    margin-top: 50px;
                    padding-top: 30px;
                    border-top: 1px solid #e5eff5;
                    text-align: center;
                    font-size: 14px;
                    color: #738a94;
                }}

                .footer a {{
                    color: #3eb0ef;
                    text-decoration: none;
                }}

                /* Reading time */
                .reading-time {{
                    font-size: 14px;
                    color: #738a94;
                    margin-bottom: 30px;
                }}

                /* Links */
                a {{
                    color: #3eb0ef;
                    text-decoration: none;
                }}

                /* Responsive */
                @media (max-width: 600px) {{
                    .container {{
                        padding: 20px 15px;
                    }}

                    .post-title {{
                        font-size: 28px;
                    }}
                }}

                /* Add these new styles after the header styles */
                .view-online-link {{
                    display: block;
                    text-align: center;
                    margin-bottom: 30px;
                    color: #738a94;
                    font-size: 13px;
                    text-decoration: none;
                }}

                .view-online-link:hover {{
                    color: #3eb0ef;
                }}
            </style>
        </head>
        <body>
            <div class="container">
                <div class="header">
                    <h1>{site_title}</h1>
                </div>

                <a href="{post_url}" class="view-online-link">View this post in your browser →</a>

                <article>
                    <h1 class="post-title">{post_title}</h1>
                    
                    <div class="reading-time">
                        {reading_time} min read
                    </div>

                    {feature_image_html}

                    <div class="author-info">
                        {author_image_html}
                        <div>
                            <div class="author-name">{author_name}</div>
                            {author_bio_html}
                        </div>
                    </div>

                    <div class="content">
                        {post_content}
                    </div>
                </article>

                <div class="footer">
                    <p>You received this email because you signed up for updates from {site_title}.</p>
                    <p>
                        <a href="{subscription_link}">Manage subscription</a> • 
                        <a href="{unsubscribe_link}">Unsubscribe</a>
                    </p>
                </div>
            </div>
        </body>
        </html>
    "#,
        site_title = template.site.title,
        post_url = template.post.url,
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
        post_content = template.post.html,
        subscription_link = template.newsletter.subscription_link,
        unsubscribe_link = template.newsletter.unsubscribe_link
    );

    Ok(html)
}
