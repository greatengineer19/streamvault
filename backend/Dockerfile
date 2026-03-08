# ── Build stage ───────────────────────────────────────────────────────────────
FROM rust:slim as builder

# musl-tools is required to compile a fully static binary targeting musl libc.
# Without this, the binary is dynamically linked against glibc (e.g. GLIBC_2.38),
# which caused a runtime crash on Railway:
#   "version `GLIBC_2.38' not found (required by streamvault-api)"
# Railway's runtime environment uses an older glibc than what debian:bookworm-slim
# compiles against, causing a version mismatch at container startup.
# Compiling to x86_64-unknown-linux-musl produces a static binary with zero
# glibc dependency, so it runs on any Linux environment regardless of glibc version.
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

# Cache dependencies first
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm src/main.rs

# Build the real app
COPY src ./src
RUN touch src/main.rs && cargo build --release --target x86_64-unknown-linux-musl

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM alpine:3.19

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/streamvault-api /usr/local/bin/streamvault-api

EXPOSE 8080

CMD ["streamvault-api"]