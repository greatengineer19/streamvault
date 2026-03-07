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

### Backend
```bash
cd backend
cp .env.example .env   # fill in your values
cargo run
# API running at http://localhost:8080
```

### Frontend
```bash
cd frontend
cp .env.example .env.local
npm install
npm run dev
# App running at http://localhost:5173
```

### R2 CORS Configuration

In your Cloudflare R2 bucket settings, add this CORS rule to allow browser uploads:

```json
[
  {
    "AllowedOrigins": ["http://localhost:5173", "https://your-vercel-app.vercel.app"],
    "AllowedMethods": ["PUT", "GET"],
    "AllowedHeaders": ["*"],
    "MaxAgeSeconds": 3600
  }
]
```

## Deployment

See [ARCHITECTURE.md](./ARCHITECTURE.md) for full deployment instructions.