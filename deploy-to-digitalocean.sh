#!/bin/bash
set -e

# Check if doctl is installed
if ! command -v doctl &> /dev/null; then
    echo "doctl is not installed. Please install the DigitalOcean CLI first."
    echo "Visit: https://docs.digitalocean.com/reference/doctl/how-to/install/"
    exit 1
fi

# Check if user is authenticated
if ! doctl account get &> /dev/null; then
    echo "Please authenticate with DigitalOcean first by running: doctl auth init"
    exit 1
fi

# Check if required environment variables are set
if [ -z "$GOOGLE_CLIENT_ID" ] || [ -z "$GOOGLE_CLIENT_SECRET" ] || [ -z "$GOOGLE_PROJECT_ID" ]; then
    echo "Please set the required environment variables:"
    echo "GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, GOOGLE_PROJECT_ID"
    echo ""
    echo "Example: export GOOGLE_CLIENT_ID=your_client_id_here"
    exit 1
fi

# Prompt for app name
read -p "Enter your app name (default: dishwashmon): " APP_NAME
APP_NAME=${APP_NAME:-dishwashmon}

# Create app spec with environment variables
cat > .do/app.yaml << EOF
name: $APP_NAME
region: nyc
services:
- name: web
  github:
    repo: $(git config --get remote.origin.url | sed 's/.*github.com[:/]\(.*\).git/\1/')
    branch: $(git rev-parse --abbrev-ref HEAD)
  dockerfile_path: Dockerfile
  http_port: 3000
  instance_count: 1
  instance_size_slug: basic-xs
  routes:
  - path: /
  envs:
  - key: GOOGLE_CLIENT_ID
    scope: RUN_TIME
    value: $GOOGLE_CLIENT_ID
    type: SECRET
  - key: GOOGLE_CLIENT_SECRET
    scope: RUN_TIME
    value: $GOOGLE_CLIENT_SECRET
    type: SECRET
  - key: GOOGLE_PROJECT_ID
    scope: RUN_TIME
    value: $GOOGLE_PROJECT_ID
    type: SECRET
  - key: HOST
    scope: RUN_TIME
    value: \${APP_DOMAIN}
  - key: REDIRECT_URI
    scope: RUN_TIME
    value: https://\${APP_DOMAIN}/auth/callback
  - key: SERVER_PORT
    scope: RUN_TIME
    value: "3000"
  - key: RUST_LOG
    scope: RUN_TIME
    value: "info"
  - key: DATA_FILE
    scope: RUN_TIME
    value: "/app/data/users.json"
EOF

echo "Creating DigitalOcean App Platform app..."
doctl apps create --spec .do/app.yaml

echo "Deployment initiated. Check your DigitalOcean dashboard for progress."
echo "Once deployed, make sure to update your Google OAuth credentials with the correct redirect URI."
echo "The redirect URI will be https://YOUR-APP-DOMAIN/auth/callback"