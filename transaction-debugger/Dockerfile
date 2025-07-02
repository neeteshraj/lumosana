# Build stage
FROM rust:latest AS builder

# Install required tools for validation and building
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    curl \
    jq \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy environment configuration files and validation scripts first
COPY .env.docker ./
COPY .env.example ./
COPY scripts/ ./scripts/

# Simple validation: Check required files exist
RUN echo "� Validating Docker environment..." && \
    test -f .env.docker || (echo "❌ Missing .env.docker file!" && exit 1) && \
    test -f .env.example || (echo "❌ Missing .env.example file!" && exit 1) && \
    echo "✅ Basic environment validation passed"

# Copy dependency files for better caching
COPY Cargo.toml Cargo.lock build.rs ./
COPY proto/ ./proto/

# Create a dummy main.rs and lib.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release
RUN rm src/main.rs src/lib.rs

# Copy source code and rebuild with validation
COPY src/ ./src/

# Final validation before building the application
RUN echo "🔍 Final validation before building..." && \
    echo "✅ Environment files validated successfully"

# Build the application with Docker environment
ENV ENVIRONMENT=docker
RUN echo "🏗️ Building application for Docker environment..." && \
    touch src/main.rs && \
    cargo build --release && \
    echo "✅ Build completed successfully!"

# Validate the built binary
RUN echo "🧪 Testing the built binary..." && \
    ./target/release/main --version && \
    echo "✅ Binary validation successful!"

# Runtime stage
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder stage
COPY --from=builder /app/target/release/main /usr/local/bin/transaction-debugger

# Copy the environment configuration for runtime
COPY --from=builder /app/.env.docker /app/.env.docker

# Set working directory to /app so the .env.docker file can be found
WORKDIR /app

# Set the environment variable so the app knows to look for .env.docker
ENV ENVIRONMENT=docker

# Create non-root user
RUN useradd -r -s /bin/false appuser
USER appuser

EXPOSE 8080 50051

CMD ["transaction-debugger"]
