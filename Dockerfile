# ==========================================
# 1. Build Stage
# ==========================================
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests for caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source files to cache compiled dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the release binary
RUN touch src/main.rs src/lib.rs && \
    cargo build --release --bin daiana

# ==========================================
# 2. Runtime Stage
# ==========================================
FROM debian:bookworm-slim AS runner

# Install runtime dependencies (SSL certificates & OpenSSL)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN groupadd -g 10001 daiana && \
    useradd -u 10001 -g daiana -s /bin/sh -M daiana

# Copy the compiled binary
COPY --from=builder /app/target/release/daiana /usr/local/bin/daiana

# Set environment defaults
ENV HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=info \
    MAX_CLIENTS_ON_CHANNEL=5 \
    CHANNEL_TIMEOUT=30

EXPOSE 8080

USER daiana:daiana

ENTRYPOINT ["daiana"]
