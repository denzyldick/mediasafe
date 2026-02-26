FROM rust:1.88-slim-bookworm as builder

WORKDIR /usr/src/app
COPY . .

# Build for release
RUN cargo build --release

# Create a lightweight runtime image
FROM debian:bookworm-slim

# Install OpenSSL required by some Rust crates
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/signaling-server /app/signaling-server

# Expose the default WebSocket port
EXPOSE 9489

# Run the server
CMD ["./signaling-server"]
