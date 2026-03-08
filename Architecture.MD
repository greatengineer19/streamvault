# StreamVault — Architecture & Design Decisions

## Overview

StreamVault is a minimal private video streaming service built for fast time-to-stream, horizontal scalability, and low operational cost. Users upload videos anonymously and receive a shareable link they can send to anyone.

---

## Stack

| Layer | Technology | Deployed on |
|---|---|---|
| Backend API | Rust + Axum | Fly.io |
| Frontend | Svelte + SvelteKit | Vercel |
| Video Storage | Cloudflare R2 | Cloudflare |
| Database | PostgreSQL | Fly.io Managed Postgres |

---

## Core Architecture

```
Browser (Vercel)
    │
    │ 1. POST /api/upload/init  → get presigned PUT URL + video_id
    │ 2. PUT file directly to R2 (presigned URL, browser → R2, no server touch)
    │ 3. POST /api/upload/complete/:id  → marks video ready in Postgres
    │
    ▼
Rust/Axum API (Fly.io) ←→ PostgreSQL (Fly.io)
    │
    │ 4. GET /api/video/:id/stream  → returns presigned GET URL
    │
    ▼
Cloudflare R2  ←──── Browser streams directly via Range Requests
```

---

## Decision 1: No Transcoding

**The requirement says time-to-stream matters more than quality.**

Transcoding (converting video to HLS/DASH with multiple quality tiers) takes minutes for large files and requires significant server resources. We skip it entirely.

Instead, we accept only browser-native formats (MP4/H.264, WebM, OGG, MOV). The browser's native `<video>` element plays these directly without any server-side processing. Time-to-stream = upload time + ~1 second to mark as ready.

**Trade-off acknowledged**: No adaptive quality switching. Viewers on slow connections may buffer. This is an acceptable trade-off given the explicit requirement priority.

---

## Decision 2: Presigned Direct Upload to R2

The browser uploads the video file **directly to Cloudflare R2** using a time-limited presigned PUT URL. Our API server never proxies the video bytes.

**Why this matters:**
- A 1GB file upload does not consume server memory or bandwidth
- The API server stays lean and horizontally scalable
- Upload speed is limited only by the user's connection and R2, not our server

The flow: browser asks our API for a presigned URL → API generates it and stores a `pending` DB record → browser PUTs directly to R2 → browser notifies API → API marks video `ready`.

---

## Decision 3: Presigned GET URLs for Streaming

When a viewer visits `/watch/:id`, SvelteKit calls `/api/video/:id/stream` which returns a **time-limited presigned GET URL** pointing directly to R2. The browser's `<video>` element streams from R2 directly.

**Why not proxy through our server?**
- R2 has zero egress fees — streaming through our server would add latency and cost
- R2 natively supports HTTP `Range` requests, which the browser uses automatically for seeking and buffered streaming
- Our server has zero load from video bytes regardless of how many concurrent viewers there are

Presigned URLs expire after 4 hours — long enough for any viewing session.

---

## Decision 4: PostgreSQL on Fly.io

We use Postgres (via SQLx in Rust) for video metadata. The schema is intentionally minimal:

```sql
videos (
  id          UUID PRIMARY KEY,   -- also the access token / shareable ID
  filename    TEXT,
  size_bytes  BIGINT,
  mime_type   TEXT,
  r2_key      TEXT,               -- path within R2 bucket
  status      TEXT,               -- 'pending' | 'ready'
  created_at  TIMESTAMPTZ
)
```

The UUID serves as the only access control mechanism — knowing the UUID is the only requirement to watch a video. This satisfies the "anonymous upload" requirement without any auth system.

---

## Decision 5: Stateless API for Horizontal Scaling

The Rust API server holds **no state**. All persistent state lives in Postgres (metadata) and R2 (video bytes). This means:

- Any number of Fly.io instances can run simultaneously
- No sticky sessions, no shared memory, no distributed cache needed
- Scaling is `fly scale count N` — that's it

---

## Playback Performance

Consistent playback regardless of file size is achieved via HTTP Range Requests:

- The browser's `<video>` element automatically requests video in chunks using the `Range` header
- R2 responds with `206 Partial Content` for each chunk
- The server is never in the critical path for streaming
- A 100MB and a 1GB file behave identically — the browser always just fetches the next chunk

---

## Cost Estimate (light usage)

| Component | Monthly cost |
|---|---|
| Cloudflare R2 storage (10GB) | **$0** (free) |
| Cloudflare R2 egress | **$0** (free) |
| Fly.io API (1 shared CPU, 256MB, auto-stop) | ~$1.94 |
| Fly.io Postgres (shared) | ~$3.00 |
| Vercel (SvelteKit frontend) | $0 (free tier) |
| **Total** | **~$5/month** |

R2's free egress is the key cost driver here. Serving video through any other cloud storage provider (S3, GCS) would add significant bandwidth costs at scale.

---

## What Was Intentionally Left Out

| Feature | Reason omitted |
|---|---|
| HLS/DASH adaptive streaming | Breaks time-to-stream requirement |
| User accounts / authentication | Requirements allow anonymous upload |
| Video thumbnails | Not required; straightforward to add |
| Link expiry | Not required; UUID security is sufficient for a private service |
| Multipart upload | Single PUT handles up to 5GB on R2; 1GB limit is well within range |
| Format conversion (MKV → MP4) | Would require transcoding; rejected above |

---

## Deployment

### Backend (Fly.io)
```bash
cd backend
fly launch          # first time
fly secrets set DATABASE_URL=... R2_ACCOUNT_ID=... R2_ACCESS_KEY_ID=... R2_SECRET_ACCESS_KEY=... R2_BUCKET=... PUBLIC_URL=https://streamvault-api.fly.dev
fly deploy
```

### Frontend (Vercel)
```bash
cd frontend
# Set PUBLIC_API_URL=https://streamvault-api.fly.dev in Vercel project settings
vercel deploy
```

### Cloudflare R2 Setup
1. Create a bucket named `streamvault-videos` in Cloudflare R2
2. Create an R2 API token with Object Read & Write permissions
3. Set the bucket CORS policy to allow PUT from your frontend origin