#!/bin/bash

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Transaction Debugger Local Runner${NC}"
echo "=================================="

# Step 1: Validate environment configuration
echo -e "\n${YELLOW}Step 1: Validating environment configuration...${NC}"
if ! "$SCRIPT_DIR/env-manager.sh" validate --no-deps-check; then
    echo -e "${RED}❌ Environment validation failed!${NC}"
    echo "Please fix the configuration issues before running the application."
    exit 1
fi
echo -e "${GREEN}✓ Environment validation passed${NC}"

# Step 2: Build the application
echo -e "\n${YELLOW}Step 2: Building transaction debugger...${NC}"
cargo build --release

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Build successful!${NC}"
    
    # Step 3: Show current environment configuration
    echo -e "\n${YELLOW}Step 3: Current environment configuration:${NC}"
    "$SCRIPT_DIR/env-manager.sh" start --show-config --no-deps-check
    
    echo -e "\n${YELLOW}Step 4: Starting the service...${NC}"
    echo -e "${GREEN}HTTP API will be available on http://localhost:8080${NC}"
    echo -e "${GREEN}gRPC API will be available on localhost:50051${NC}"
    echo -e "${GREEN}Health check: http://localhost:8080/health${NC}"
    echo -e "${GREEN}Swagger UI: http://localhost:8080/swagger-ui/${NC}"
    echo ""
    echo -e "${YELLOW}Press Ctrl+C to stop the service${NC}"
    echo ""
    
    RUST_LOG=info cargo run --release
else
    echo -e "${RED}❌ Build failed!${NC}"
    exit 1
fi
