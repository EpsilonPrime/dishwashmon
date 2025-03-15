FROM rust:1.76-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy manifests and build dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Copy the source code and build the application
COPY . .
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/dishwashmon /app/dishwashmon

# Create a directory for configuration files
RUN mkdir -p /app/config

# Expose the web server port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV SERVER_PORT=3000

# Run the application
CMD ["/app/dishwashmon"]