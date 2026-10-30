# syntax=docker/dockerfile:1

# ---- build ----
FROM rust:latest AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
COPY locales ./locales
RUN cargo build --release

# ---- runtime ----
FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends android-tools-adb ca-certificates \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/alejandradb /usr/local/bin/alejandradb
ENTRYPOINT ["alejandradb"]
