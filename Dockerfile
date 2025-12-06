# ============================
# 1. Builder image
# ============================
FROM debian:bookworm-slim AS builder

# Install system dependencies (customize if your crates need libs)
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl build-essential gcc g++ make pkg-config python3 \
    libssl-dev zlib1g-dev libzstd-dev liblz4-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Install Rust toolchain
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Create app directory
WORKDIR /app

# Cache dependency builds
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release

# ============================
# 2. Runtime image
# ============================
FROM debian:bookworm-slim

# OS dependencies your runtime might need
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates openssl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from builder
COPY --from=builder /app/target/release/rdfetcher ./rdfetcher

# Copy configuration
COPY Config.toml ./Config.toml
COPY .env .env

# Expose ports (change if needed)
# EXPOSE 8080

# Run the binary
CMD ["./rdfetcher"]
