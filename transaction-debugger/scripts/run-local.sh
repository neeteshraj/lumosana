#!/bin/bash

echo "Building transaction debugger..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "Build successful! Starting the service..."
    echo "HTTP API will be available on http://localhost:8080"
    echo "gRPC API will be available on localhost:50051"
    echo "Health check: http://localhost:8080/health"
    echo "Swagger UI: http://localhost:8080/swagger-ui/"
    echo ""
    echo "Press Ctrl+C to stop the service"
    echo ""
    
    RUST_LOG=info cargo run --release
else
    echo "Build failed!"
    exit 1
fi
