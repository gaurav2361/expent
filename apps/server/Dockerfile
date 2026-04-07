FROM rust:1.85-slim AS builder

WORKDIR /app

# Install dependencies needed for compiling
RUN apt-get update && apt-get install -y perl make pkg-config libssl-dev gcc && rm -rf /var/lib/apt/lists/*

# Copy the entire workspace to resolve crates/* properly
COPY . .

# Build the server binary using docker cache mounts to radically speed up rebuilds
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin server && \
    cp target/release/server /tmp/server

FROM debian:bookworm-slim AS runner
WORKDIR /app

# Install runtime dependencies like openssl and ca-certificates
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the cached tmp location
COPY --from=builder /tmp/server /usr/local/bin/

# Make sure we run with info logs by default
ENV RUST_LOG=info
ENV PORT=7878

EXPOSE 7878

CMD ["server"]
