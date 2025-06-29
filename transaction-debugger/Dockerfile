# Build stage
FROM rust:1.77 as builder

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock build.rs ./
COPY proto/ ./proto/

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Copy source code and rebuild
COPY src/ ./src/
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder stage
COPY --from=builder /app/target/release/main /usr/local/bin/transaction-debugger

# Create non-root user
RUN useradd -r -s /bin/false appuser
USER appuser

EXPOSE 8080 50051

CMD ["transaction-debugger"]
