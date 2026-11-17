# EKPI Platform - governance_web image (correctness fix only)
#
# Stage 1 deploys the Windows native binary, not this image. The image is kept
# buildable so it no longer targets the `apps/web` skeleton, but it is not a
# supported Stage 1 deployment artifact: the workbench binds loopback only and
# refuses non-loopback addresses until a Stage 2 network decision exists.
#
# Strict Zero-JS: no JavaScript in the final image.

# Build stage
FROM rust:1.99.1-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --locked --bin governance_web

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/governance_web /app/governance_web
COPY --from=builder /app/platform/data /app/data
ENV EKPI_DATA_DIR=/app/data
ENV EKPI_BIND_IP=127.0.0.1
ENV EKPI_WEB_PORT=3000
EXPOSE 3000
CMD ["/app/governance_web"]