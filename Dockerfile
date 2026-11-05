# EKPI Platform - Multi-stage Docker build
# Strict Zero-JS: No JavaScript in final image

# Build stage
FROM rust:1.99.1-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin ekp_web

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/ekp_web /app/ekp_web
COPY --from=builder /app/apps/web/static /app/static
COPY --from=builder /app/apps/web/templates /app/templates
EXPOSE 3000
ENV RUST_LOG=info
CMD ["/app/ekp_web"]