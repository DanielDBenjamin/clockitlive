FROM rustlang/rust:nightly as builder

# Install system dependencies
RUN apt-get update && apt-get install -y binaryen npm && rm -rf /var/lib/apt/lists/*
RUN npm install -g sass

WORKDIR /app

# Install required Rust targets and tooling up front so this layer stays cached
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-bindgen-cli --version 0.2.103

# Copy project sources
COPY . .

# Build
RUN chmod +x build.sh && ./build.sh

# Verify build artifacts
RUN ls -la target/site/
RUN ls -la target/site/pkg/ || echo "WARNING: pkg directory not found"

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy built artifacts
COPY --from=builder /app/clock-it /app/clock-it
COPY --from=builder /app/target/site /app/target/site
COPY --from=builder /app/start.sh /app/start.sh
COPY --from=builder /app/Cargo.toml /app/Cargo.toml

# Make executable
RUN chmod +x /app/clock-it /app/start.sh

# Create directory for SQLite database
RUN mkdir -p /app/data

# Verify files exist
RUN ls -la /app/target/site/
RUN ls -la /app/target/site/pkg/ || echo "WARNING: pkg directory not found in final image"

EXPOSE 3000

# Set Railway-specific environment variables
ENV CLOCK_IT_USE_TLS=false
ENV LEPTOS_SITE_ROOT=/app/target/site
ENV LEPTOS_SITE_PKG_DIR=pkg

CMD ["./start.sh"]
