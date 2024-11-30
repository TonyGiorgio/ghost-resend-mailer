# Ghost Resend Mailer

A service that listens for new Ghost blog posts and emails them to your subscribers using Resend.

## Setup

1. Create a `.env` file with the following variables:
    ```env
    GHOST_URL=http://your-ghost-blog.com
    GHOST_ADMIN_ID=first_part_of_ghost_key
    GHOST_ADMIN_SECRET=second_part_of_ghost_key
    WEBHOOK_SECRET=your_webhook_secret
    RESEND_API_KEY=your_resend_api_key
    FROM_EMAIL=your-blog@yourdomain.com
    PORT=3000
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
     - Secret: Generate a random string openssl
     - Copy this same secret to your `.env` file's `WEBHOOK_SECRET`

4. Get your Resend API key:
   - Sign up at [resend.com](https://resend.com)
   - Go to the API Keys section
   - Create a new API key
   - Copy it to your `.env` file's `RESEND_API_KEY`

5. Run the service:
    ```bash
    cargo run
    ```
