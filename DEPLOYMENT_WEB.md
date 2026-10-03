# Proofer — Web Production & Deployment Guide

This guide details how to build, containerize, and deploy the **Proofer** theorem-proving workstation to the web for production use.

---

## 1. Local Production Test with Docker

### Build the Image
```bash
docker build -t proofer:latest .
```

### Run the Container
```bash
docker run -d --name proofer-app -p 8086:8086 proofer:latest
```

Open `http://localhost:8086` in your browser.

### Using Docker Compose
```bash
docker compose up -d
```

---

## 2. Deploy to Fly.io (Recommended — Edge Hosting)

[Fly.io](https://fly.io) runs Docker containers at the network edge with automatic TLS/HTTPS:

1. Install the Fly CLI:
   ```bash
   curl -L https://fly.io/install.sh | sh
   ```
2. Authenticate:
   ```bash
   fly auth login
   ```
3. Launch & Deploy using the preconfigured `fly.toml`:
   ```bash
   fly launch --name proofer-workstation
   fly deploy
   ```
Your app will be live at `https://proofer-workstation.fly.dev` with automatic HTTPS certificates and global low latency.

---

## 3. Deploy to Google Cloud Run (Serverless Container)

[Google Cloud Run](https://cloud.google.com/run) scales containers automatically (including scale to zero when idle):

1. Authenticate with Google Cloud:
   ```bash
   gcloud auth login
   gcloud config set project YOUR_PROJECT_ID
   ```
2. Build & Deploy in one step:
   ```bash
   gcloud run deploy proofer \
     --source . \
     --platform managed \
     --region us-central1 \
     --allow-unauthenticated \
     --port 8086 \
     --memory 256Mi
   ```
Cloud Run automatically provisions a secure `https://...run.app` URL.

---

## 4. Deploy to Railway or Render

### Railway:
1. Push your repository to GitHub.
2. In [Railway.app](https://railway.app), click **New Project** → **Deploy from GitHub repo**.
3. Railway automatically detects `Dockerfile`, builds the multi-stage image, and provisions a public domain.

### Render:
1. In [Render.com](https://render.com), create a **New Web Service**.
2. Select your GitHub repository.
3. Choose environment **Docker**, set internal port to `8086`, and click **Deploy**.

---

## 5. Deploy to a Linux VPS (Ubuntu / Debian + Caddy)

If deploying to a self-managed server (DigitalOcean, Linode, AWS EC2, Hetzner):

1. Clone and build the release binary:
   ```bash
   git clone https://github.com/your-username/Proofer.git
   cd Proofer/frontend && npm ci && npm run build
   cd ../proof && cargo build --release
   ```
2. Create a systemd service (`/etc/systemd/system/proofer.service`):
   ```ini
   [Unit]
   Description=Proofer Verification Server
   After=network.target

   [Service]
   Type=simple
   User=www-data
   WorkingDirectory=/var/www/proofer/proof
   ExecStart=/var/www/proofer/proof/target/release/proof --server
   Restart=always
   Environment=PORT=8086
   Environment=HOST=127.0.0.1

   [Install]
   WantedBy=multi-user.target
   ```
3. Start the service:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable --now proofer
   ```
4. Point [Caddy](https://caddyserver.com) to it for automatic SSL:
   ```caddy
   # /etc/caddy/Caddyfile
   proofer.yourdomain.com {
       encode zstd gzip
       reverse_proxy 127.0.0.1:8086
   }
   ```
5. Reload Caddy:
   ```bash
   sudo systemctl reload caddy
   ```

---

## Architecture Summary
- **Frontend**: React 19 + TypeScript + Vite, precompiled to static assets.
- **Backend**: Rust release binary handling incremental verification and static file delivery.
- **Image Size**: Under 30 MB (Alpine base, unprivileged user, stripped binary).
- **Security**: No external runtime dependencies, non-root execution, CORS-enabled, dynamic origin routing.
