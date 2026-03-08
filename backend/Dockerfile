# -- Build stage ---------
FROM rust:slim as builder

WORKDIR /app

# Cache dependencies first
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Build the real app
COPY src ./src
RUN touch src/main.rs && cargo build --release

# -- Runtime stage -------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/streamvault-api /usr/local/bin/streamvault-api

EXPOSE 8079

CMD ["streamvault-api"]