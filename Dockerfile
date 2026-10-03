# ==============================================================================
# STAGE 1: Build Frontend Assets with Node.js & Vite
# ==============================================================================
FROM node:22-alpine AS frontend-builder
WORKDIR /app/frontend

COPY frontend/package*.json ./
RUN npm ci

COPY frontend/ ./
RUN npm run build

# ==============================================================================
# STAGE 2: Build Rust Backend Release Binary
# ==============================================================================
FROM rust:1.85-alpine AS backend-builder
WORKDIR /app/proof

RUN apk add --no-cache musl-dev

COPY proof/Cargo.toml proof/Cargo.lock ./
# Create dummy source to cache downloaded dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY proof/src ./src
COPY proof/tests ./tests
# Copy compiled frontend into static directory for embedding/serving
COPY --from=frontend-builder /app/frontend/dist ./static

RUN touch src/main.rs && cargo build --release --locked

# ==============================================================================
# STAGE 3: Minimal, Hardened Production Image (< 30 MB)
# ==============================================================================
FROM alpine:3.21 AS runner
WORKDIR /app

# Non-root user for security
RUN addgroup -S proofer && adduser -S proofer -G proofer
RUN apk add --no-cache ca-certificates tzdata

COPY --from=backend-builder /app/proof/target/release/proof /app/proofer
COPY --from=backend-builder /app/proof/static /app/static

ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=8086

USER proofer
EXPOSE 8086

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -q --spider http://127.0.0.1:8086/ || exit 1

ENTRYPOINT ["/app/proofer", "--server"]
