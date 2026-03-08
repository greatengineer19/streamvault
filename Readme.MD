# StreamVault

> Minimal private video streaming service — upload once, stream instantly.

Upload a video file (up to 1GB) and get a shareable link. No accounts required. The browser uploads directly to Cloudflare R2; the API server never touches video bytes.

**Live demo:** https://streamvault-lilac.vercel.app

---

## Stack

| Layer | Technology | Hosted on |
|---|---|---|
| Frontend | SvelteKit | Vercel (free) |
| Backend API | Rust + Axum | Railway (free trial) |
| Video Storage | Cloudflare R2 | Cloudflare (free egress) |
| Database | PostgreSQL | Neon (free tier) |

---

## Project Structure

```
streamvault/
├── backend/               # Rust/Axum API
│   ├── src/
│   │   ├── main.rs        # Server entry, router setup
│   │   ├── config.rs      # Env var loading
│   │   ├── routes.rs      # HTTP handlers
│   │   ├── models.rs      # Shared data types
│   │   ├── db.rs          # Postgres connection + migrations
│   │   ├── r2.rs          # Cloudflare R2 / presigned URLs
│   │   ├── error.rs       # Unified error handling
│   │   └── tests.rs       # Unit tests
│   ├── Dockerfile
│   ├── fly.toml
│   └── .env.example
│
├── frontend/              # SvelteKit app
│   ├── src/
│   │   ├── app.html
│   │   ├── app.css
│   │   ├── lib/
│   │   │   └── api.ts     # API client
│   │   └── routes/
│   │       ├── +layout.svelte
│   │       ├── +page.svelte          # Upload page (/)
│   │       └── watch/[id]/
│   │           ├── +page.ts          # Load function
│   │           └── +page.svelte      # Watch page (/watch/:id)
│   ├── svelte.config.js
│   ├── vite.config.ts
│   ├── package.json
│   └── .env.example
│
├── docker-compose.yml     # Local Postgres
├── ARCHITECTURE.md        # Design decisions
└── README.md
```

---

## Prerequisites

### Rust

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Windows: download the installer from [rustup.rs](https://rustup.rs).

```bash
rustc --version && cargo --version  # verify
```

### Node.js (v18+)

```bash
# macOS / Linux — via nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
nvm install 20 && nvm use 20
```

Windows: download the LTS installer from [nodejs.org](https://nodejs.org).

```bash
node --version && npm --version  # verify
```

---

## Local Development

### 1. Clone

```bash
git clone https://github.com/your-org/streamvault.git
cd streamvault
```

### 2. Start PostgreSQL

```bash
docker compose up -d
```

Starts Postgres 16 at `localhost:5432`. The backend auto-creates the schema on first boot.

```bash
docker compose down      # stop
docker compose down -v   # stop + wipe data
```

### 3. Set up Cloudflare R2

1. Create a bucket named `streamvault-videos` in the [Cloudflare Dashboard](https://dash.cloudflare.com).
2. Generate an API token under **R2 → Manage R2 API Tokens** (Object Read & Write).
3. Set the CORS policy under **R2 → your bucket → Settings → CORS Policy**:

```json
[
  {
    "AllowedOrigins": ["http://localhost:5173"],
    "AllowedMethods": ["PUT", "GET"],
    "AllowedHeaders": ["*"],
    "MaxAgeSeconds": 3600
  }
]
```

When deploying, add your production frontend URL to `AllowedOrigins`.

### 4. Configure the backend

```bash
cd backend
cp .env.example .env
```

Fill in your R2 credentials:

```env
R2_ACCOUNT_ID=your_cloudflare_account_id
R2_ACCESS_KEY_ID=your_r2_access_key_id
R2_SECRET_ACCESS_KEY=your_r2_secret_access_key
R2_BUCKET=streamvault-videos
```

`DATABASE_URL` is pre-configured for the Docker Compose Postgres — no changes needed locally.

### 5. Start the backend

```bash
cargo run
```

First run compiles all dependencies (~2–3 minutes). Subsequent runs are fast.

```bash
curl http://localhost:8079/api/health  # → {"status":"ok"}
```

### 6. Start the frontend

```bash
cd frontend
cp .env.example .env.local
npm install
npm run dev
```

Open **http://localhost:5173**.

---

## Local Services

| Service | URL |
|---|---|
| Frontend | http://localhost:5173 |
| API | http://localhost:8079 |
| Postgres | localhost:5432 |
| Video storage | Cloudflare R2 (remote) |

---

## Deployment

See [ARCHITECTURE.md](./ARCHITECTURE.md) for full deployment instructions (Railway + Vercel + Neon).

---

## Code Quality

```bash
cd backend
cargo fmt -- --check   # check formatting
cargo clippy           # lint
cargo test             # run tests
```

---

## Packaging

Export the project without build artifacts or secrets:

```bash
zip -r streamvault.zip . \
  --exclude "*/target/*" \
  --exclude "*/node_modules/*" \
  --exclude "*/.env" \
  --exclude "*/.env.local" \
  --exclude "*/.svelte-kit/*" \
  --exclude "*/build/*"
```