# Ghost Resend Mailer

A service that listens for new Ghost blog posts and emails them to your subscribers using Resend.

## Setup

1. You'll need the following environment variables:
    ```env
    GHOST_URL=http://your-ghost-blog.com
    GHOST_ADMIN_ID=first_part_of_ghost_key
    GHOST_ADMIN_SECRET=second_part_of_ghost_key
    WEBHOOK_SECRET=your_webhook_secret
    RESEND_API_KEY=your_resend_api_key
    FROM_EMAIL=your-blog@yourdomain.com
    ```

2. Get your Ghost Admin API key:
   - Go to your Ghost Admin panel
   - Settings → Integrations
   - Create a new Custom Integration
   - Copy the Admin API Key

3. Create a webhook in Ghost:
   - Go to your Ghost Admin panel
   - Settings → Integrations
   - Click on your Custom Integration
   - Scroll down to Webhooks
   - Click "Add webhook"
   - Set the following:
     - Event: `Post published`
     - Target URL: `http://your-server:3000/webhook`
     - Secret: Generate a random string using openssl
     - Save this secret, you'll need it for the WEBHOOK_SECRET environment variable

4. Get your Resend API key:
   - Sign up at [resend.com](https://resend.com)
   - Go to the API Keys section
   - Create a new API key

## Development

Run the service locally:
```bash
cargo run
```

## Deployment

### Using Docker

You can pull the pre-built container from GitHub Container Registry:

```bash
# Pull the latest version
docker pull ghcr.io/tonygiorgio/ghost-resend-mailer:latest

# Or pull a specific version
docker pull ghcr.io/tonygiorgio/ghost-resend-mailer:v0.1.0
```

The container is built for x86_64 (AMD64) architecture. Docker will automatically pull the correct version for your system.

Run the container:
```bash
docker run -d \
  -p 3000:3000 \
  -e GHOST_URL=http://your-ghost-blog.com \
  -e GHOST_ADMIN_ID=your_admin_id \
  -e GHOST_ADMIN_SECRET=your_admin_secret \
  -e WEBHOOK_SECRET=your_webhook_secret \
  -e RESEND_API_KEY=your_resend_api_key \
  -e FROM_EMAIL=your-blog@yourdomain.com \
  ghcr.io/tonygiorgio/ghost-resend-mailer:latest
```

### Using Docker Compose

```yaml
version: '3.8'
services:
  mailer:
    image: ghcr.io/tonygiorgio/ghost-resend-mailer:latest
    ports:
      - "3000:3000"
    environment:
      - GHOST_URL=${GHOST_URL:?GHOST_URL is required}
      - GHOST_ADMIN_ID=${GHOST_ADMIN_ID:?GHOST_ADMIN_ID is required}
      - GHOST_ADMIN_SECRET=${GHOST_ADMIN_SECRET:?GHOST_ADMIN_SECRET is required}
      - WEBHOOK_SECRET=${WEBHOOK_SECRET:?WEBHOOK_SECRET is required}
      - RESEND_API_KEY=${RESEND_API_KEY:?RESEND_API_KEY is required}
      - FROM_EMAIL=${FROM_EMAIL:?FROM_EMAIL is required}
      - PORT=3000
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 5s
```

The service exposes the following endpoints:
- `/webhook` - Webhook endpoint for Ghost
- `/health` - Health check endpoint

The service includes graceful shutdown handling for proper container orchestration.
