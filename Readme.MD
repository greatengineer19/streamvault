# StreamVault

Minimal private video streaming service. Upload once, stream instantly.

- **Backend**: Rust + Axum → Fly.io
- **Frontend**: Svelte + SvelteKit → Vercel
- **Storage**: Cloudflare R2
- **Database**: PostgreSQL (Fly.io)

## Project Structure

```
streamvault/
├── backend/          # Rust/Axum API
│   ├── src/
│   │   ├── main.rs   # Server entry, router setup
│   │   ├── config.rs # Env var loading
│   │   ├── routes.rs # All HTTP handlers
│   │   ├── models.rs # Shared data types
│   │   ├── db.rs     # Postgres connection + migrations
│   │   ├── r2.rs     # Cloudflare R2 / presigned URLs
│   │   └── error.rs  # Unified error handling
│   ├── Dockerfile
│   ├── fly.toml
│   └── .env.example
│
├── frontend/         # SvelteKit app
│   ├── src/
│   │   ├── app.html                             # Root HTML shell (required by SvelteKit)
│   │   ├── app.css                              # Global styles
│   │   ├── lib/
│   │   │   └── api.ts                           # API client
│   │   └── routes/
│   │       ├── +layout.svelte                   # Root layout
│   │       ├── +page.svelte                     # Upload page (/)
│   │       └── watch/
│   │           └── [id]/
│   │               ├── +page.ts                 # Load function
│   │               └── +page.svelte             # Watch page (/watch/:id)
│   ├── svelte.config.js
│   ├── vite.config.ts
│   ├── package.json
│   └── .env.example
│
└── ARCHITECTURE.md   # Design decisions
```

## Prerequisites

### Installing Rust

Rust is installed via `rustup` — the official Rust toolchain manager. It installs the compiler (`rustc`), the package manager (`cargo`), and keeps everything up to date.

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen prompt — the default installation is fine. Then restart your terminal (or run `source $HOME/.cargo/env`) so the `cargo` command is available.

**Windows:**

Download and run the installer from https://rustup.rs. It will also prompt you to install the Visual Studio C++ build tools if you don't have them — accept that too.

**Verify it worked:**
```bash
rustc --version   # e.g. rustc 1.78.0
cargo --version   # e.g. cargo 1.78.0
```

You do not need to install Axum separately. Axum is a library (called a "crate" in Rust), and `cargo` downloads it automatically when you run `cargo build` or `cargo run` based on the versions listed in `Cargo.toml`.

---

### Installing Node.js (for SvelteKit)

SvelteKit requires Node.js 18 or higher. The recommended way is via `nvm` (Node Version Manager), which lets you switch Node versions easily.

**macOS / Linux:**
```bash
# Install nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash

# Restart terminal, then install Node 20 LTS
nvm install 20
nvm use 20
```

**Windows:**

Download and run the LTS installer from https://nodejs.org. This also installs `npm` automatically.

**Verify it worked:**
```bash
node --version   # e.g. v20.12.0
npm --version    # e.g. 10.5.0
```

You do not need to install Svelte or SvelteKit separately. They are listed as dependencies in `frontend/package.json` and are downloaded automatically when you run `npm install`. The `svelte.config.js` in this project already configures SvelteKit with the Vercel adapter.

---

## Local Development

### Step 1 — Start Postgres

From the project root:

```bash
docker compose up -d
```

This starts a Postgres 16 container at `localhost:5432` with database `streamvault`. To stop it:

```bash
docker compose down
```

To stop and wipe all data:

```bash
docker compose down -v
```

---

### Step 2 — Configure R2 CORS

Before the browser can upload directly to your R2 bucket, you need to allow it. In the Cloudflare Dashboard go to **R2 → your bucket → Settings → CORS Policy** and add:

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

When you deploy to Vercel later, add your Vercel URL to `AllowedOrigins` alongside localhost.

---

### Step 3 — Configure Backend

```bash
# from the project root
cd backend
cp .env.example .env
```

Open `.env` and fill in your real Cloudflare R2 credentials:

```bash
# These come from Cloudflare Dashboard → R2 → Manage R2 API Tokens
R2_ACCOUNT_ID=your_cloudflare_account_id
R2_ACCESS_KEY_ID=your_r2_access_key_id
R2_SECRET_ACCESS_KEY=your_r2_secret_access_key
R2_BUCKET=streamvault-videos
```

The `DATABASE_URL` in `.env.example` already matches the Docker Compose Postgres — no change needed there.

---

### Step 4 — Run the Backend

```bash
# still inside /backend
cargo run
```

On first run, Cargo downloads and compiles all dependencies — this takes a few minutes. Subsequent runs are fast. The server starts at `http://localhost:8079` and automatically creates the `videos` table in Postgres on boot.

Verify it's running:

```bash
curl http://localhost:8079/api/health
# {"status":"ok"}
```

---

### Step 5 — Run the Frontend

Open a new terminal tab:

```bash
# from the project root
cd frontend
cp .env.example .env.local
npm install
npm run dev
```

App is now running at `http://localhost:5173`. Open it in your browser and you should see the upload page.

---

### Full Local Stack

| Service | URL |
|---|---|
| SvelteKit frontend | http://localhost:5173 |
| Rust API | http://localhost:8079 |
| Postgres | localhost:5432 |
| Video storage | Cloudflare R2 (real) |

## Deployment

See [ARCHITECTURE.md](./ARCHITECTURE.md) for full deployment instructions.