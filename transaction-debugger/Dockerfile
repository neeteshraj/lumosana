# Build stage
FROM rust:latest as builder

# Install protobuf compiler
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock build.rs ./
COPY proto/ ./proto/

# Create a dummy main.rs and lib.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release
RUN rm src/main.rs src/lib.rs

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
