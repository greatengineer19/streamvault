# StreamVault — Architecture & Design Decisions

StreamVault is a minimal private video streaming service built for fast time-to-stream, horizontal scalability, and near-zero operational cost. Users upload videos anonymously and receive a shareable link.

---

## How It Works

```
Browser (Vercel)
    │
    ├─ 1. POST /api/upload/init       → presigned PUT URL + video_id
    ├─ 2. PUT file → Cloudflare R2    (direct, server never sees the bytes)
    ├─ 3. POST /api/upload/complete   → marks video "ready" in Postgres
    │
    ▼
Rust/Axum API (Railway) ←──→ PostgreSQL (Neon)
    │
    ├─ 4. GET /api/video/:id/stream   → presigned GET URL
    │
    ▼
Browser streams directly from Cloudflare R2 via HTTP Range Requests
```

The core constraint: **the API server never proxies video bytes.** Upload bandwidth goes browser → R2. Streaming bandwidth goes R2 → browser. The backend handles only metadata and URL generation.

---

## Requirements Coverage

### Core

| Requirement | How it's met |
|---|---|
| Upload up to 1GB | Enforced at the API layer before presigned URL is issued; R2 single PUT supports up to 5GB |
| Common video formats | MP4/H.264, WebM, OGG, MOV — all natively playable in browser `<video>` |
| Anonymous upload | No auth system; UUID is the only access token |
| Shareable link per video | `/watch/:id` — the UUID in the URL is the share link |
| Stream via browser | Presigned GET URL returned to `<video src>`, plays natively |
| Time-to-stream priority | No transcoding; upload → mark ready → stream in ~1 second |

### Bonus

| Requirement | How it's met |
|---|---|
| Consistent playback regardless of file size | HTTP Range Requests — browser fetches in chunks, seeking works at any size |
| Horizontal scaling | Stateless API; all state in Postgres + R2; scale by adding replicas |
| Cost efficiency | ~$0/month on free tiers; R2 free egress means streaming is always free |

---

## Design Decisions

### No Transcoding

Transcoding to HLS/DASH takes minutes per file and requires substantial server resources. Given that time-to-stream is the primary constraint, we skip it entirely and accept only browser-native formats. Time-to-stream becomes: upload time + ~1 second to mark the video ready.

**Trade-off:** No adaptive quality switching. Viewers on slow connections may buffer. This is acceptable given the stated priority.

### Presigned Direct Upload

The browser receives a time-limited presigned PUT URL from the API and uploads directly to R2. The backend issues the URL, stores a `pending` record in Postgres, and waits for a completion callback — nothing more.

This keeps the backend stateless and eliminates the server as an upload bottleneck. A 1GB upload consumes zero server memory or bandwidth.

### Presigned Streaming

On `/watch/:id`, the frontend calls `/api/video/:id/stream` and receives a presigned GET URL pointing directly to R2. The `<video>` element streams from R2 using HTTP Range Requests — R2 responds with `206 Partial Content` per chunk, enabling seeking at any file size with no server involvement.

Presigned URLs expire after 4 hours, which is sufficient for any viewing session.

### UUID as Access Token

The video's UUID serves as both the primary key and the only access control mechanism. Knowing the UUID is the only requirement to watch. This satisfies anonymous upload with zero auth infrastructure.

### Stateless API

The Rust/Axum server holds no in-memory state. All persistence is in Postgres (metadata) and R2 (video bytes). Horizontal scaling is a replica count change — no sticky sessions, no shared cache, no coordination between instances.

---

## Data Model

```sql
videos (
  id          UUID PRIMARY KEY,   -- also the share token
  filename    TEXT,
  size_bytes  BIGINT,
  mime_type   TEXT,
  r2_key      TEXT,               -- object path within the R2 bucket
  status      TEXT,               -- 'pending' | 'ready'
  created_at  TIMESTAMPTZ
)
```

Only metadata is stored — never video bytes.

---

## Cost (Light Usage)

| Service | Plan | Cost |
|---|---|---|
| Vercel | Free tier | $0 |
| Railway | Trial (2 vCPU, 1GB) | $0 |
| Cloudflare R2 | Free tier + free egress | $0 |
| Neon | Free tier (0.5GB, 100 compute hrs) | $0 |
| **Total** | | **$0–5 / month** |

R2's permanently free egress is the key cost driver. Serving video through S3 or GCS would add significant bandwidth costs at scale. Streaming here is free regardless of view count.

---

## What Was Left Out

| Feature | Reason |
|---|---|
| HLS/DASH adaptive streaming | Conflicts with time-to-stream requirement |
| User accounts / auth | Requirements allow anonymous upload; UUID access is sufficient |
| Video thumbnails | Not required; straightforward to add later |
| Link expiry | Not required; UUID is sufficient for a private service |
| Multipart upload | Single PUT handles up to 5GB on R2; 1GB limit is well within range |
| MKV / AVI support | Not natively playable in `<video>` without transcoding |

---

## Deployment

### Backend (Railway)

```bash
cd backend
# Set environment variables in Railway dashboard:
# DATABASE_URL, R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_BUCKET, PUBLIC_URL
railway up
```

### Frontend (Vercel)

```bash
cd frontend
# Set PUBLIC_API_URL=https://your-api.railway.app in Vercel project settings
vercel deploy
```

### Cloudflare R2

1. Create bucket `streamvault-videos`
2. Create API token with Object Read & Write permissions
3. Add production frontend URL to bucket CORS policy `AllowedOrigins`