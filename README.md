# Dishwashmon

A service for monitoring dishwashers using Google Nest cameras.

## Features

- OAuth2 integration with Google Nest API
- Camera discovery and selection
- Web UI for managing monitored cameras
- Event detection for dishwasher state changes

## Prerequisites

- Rust toolchain (version 1.76 or higher)
- Google Cloud Platform account with:
  - Nest API access
  - OAuth2 credentials
- Docker and Docker Compose (for containerized deployment)

## Local Development

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/dishwashmon.git
   cd dishwashmon
   ```

2. Create a `.env` file with your Google API credentials:
   ```
   GOOGLE_CLIENT_ID=your_client_id_here
   GOOGLE_CLIENT_SECRET=your_client_secret_here
   GOOGLE_PROJECT_ID=your_project_id_here
   REDIRECT_URI=http://localhost:3000/auth/callback
   SERVER_PORT=3000
   ```

3. Build and run the application:
   ```bash
   cargo run
   ```

4. Open your browser at [http://localhost:3000](http://localhost:3000)

## Deploying to DigitalOcean

### Option 1: App Platform (Recommended)

1. Fork this repository to your GitHub account

2. Log in to your DigitalOcean account and go to the App Platform

3. Click "Create App" and select your GitHub repository

4. Configure the app:
   - Choose the "Dockerfile" deployment type
   - Set the necessary environment variables:
     - `GOOGLE_CLIENT_ID`
     - `GOOGLE_CLIENT_SECRET`
     - `GOOGLE_PROJECT_ID`
     - `HOST` (will be set automatically)
     - `REDIRECT_URI` (should be your app URL + `/auth/callback`)

5. Deploy the application

Alternatively, you can use the DigitalOcean CLI:

```bash
doctl apps create --spec .do/app.yaml
```

### Option 2: Docker on a Droplet

1. Create a Droplet with Docker pre-installed

2. SSH into your Droplet

3. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/dishwashmon.git
   cd dishwashmon
   ```

4. Create a `.env` file with your Google API credentials:
   ```
   GOOGLE_CLIENT_ID=your_client_id_here
   GOOGLE_CLIENT_SECRET=your_client_secret_here
   GOOGLE_PROJECT_ID=your_project_id_here
   HOST=your_domain_or_ip
   REDIRECT_URI=https://your_domain_or_ip/auth/callback
   ```

5. Start the application with Docker Compose:
   ```bash
   docker-compose up -d
   ```

6. Set up Nginx as a reverse proxy (recommended for HTTPS):
   ```bash
   apt-get update
   apt-get install -y nginx certbot python3-certbot-nginx
   
   # Configure Nginx
   cat > /etc/nginx/sites-available/dishwashmon << 'EOF'
   server {
       listen 80;
       server_name your_domain.com;
       
       location / {
           proxy_pass http://localhost:3000;
           proxy_set_header Host $host;
           proxy_set_header X-Real-IP $remote_addr;
       }
   }
   EOF
   
   ln -s /etc/nginx/sites-available/dishwashmon /etc/nginx/sites-enabled/
   nginx -t
   systemctl restart nginx
   
   # Set up HTTPS
   certbot --nginx -d your_domain.com
   ```

7. Access your application at https://your_domain.com

## Configuration Options

| Environment Variable | Description | Default |
|---------------------|-------------|---------|
| `GOOGLE_CLIENT_ID` | Google OAuth client ID | (required) |
| `GOOGLE_CLIENT_SECRET` | Google OAuth client secret | (required) |
| `GOOGLE_PROJECT_ID` | Google Cloud project ID | (required) |
| `HOST` | Host name for the application | localhost |
| `SERVER_PORT` | Port to run the server on | 3000 |
| `REDIRECT_URI` | OAuth redirect URI | http://localhost:3000/auth/callback |
| `RUST_LOG` | Logging level | info |

## License

MIT