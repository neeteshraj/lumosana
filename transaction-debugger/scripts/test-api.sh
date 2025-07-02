#!/bin/bash

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

if [ -z "$1" ]; then
    echo -e "${BLUE}Transaction Debugger API Test${NC}"
    echo "============================="
    echo ""
    echo "Usage: $0 <transaction_signature> [rpc_url]"
    echo "Example: $0 5VfZwMZgXJzBSTwSGVLrr2uWy1o1nWfmP1JMW8rGfVGJjE3HvMqJjKwKJ2wWy1o1nWfmP1JMW8rGfVGJ"
    echo ""
    echo "API Documentation:"
    echo "  Swagger UI: http://localhost:8080/swagger-ui/"
    echo "  OpenAPI JSON: http://localhost:8080/api-docs/openapi.json"
    exit 1
fi

SIGNATURE=$1
RPC_URL=${2:-"https://api.mainnet-beta.solana.com"}
BASE_URL=${BASE_URL:-"http://localhost:8080"}

echo -e "${BLUE}Transaction Debugger API Test${NC}"
echo "============================="

# Step 1: Validate environment configuration
echo -e "\n${YELLOW}Step 1: Validating environment configuration...${NC}"
if ! "$SCRIPT_DIR/env-manager.sh" validate --no-deps-check; then
    echo -e "${RED}❌ Environment validation failed!${NC}"
    echo "Please fix the configuration issues before testing the API."
    exit 1
fi
echo -e "${GREEN}✓ Environment validation passed${NC}"

# Step 2: Show test parameters
echo -e "\n${YELLOW}Step 2: Test Parameters${NC}"
echo "Signature: $SIGNATURE"
echo "RPC URL: $RPC_URL"
echo "API Base URL: $BASE_URL"
echo "Swagger UI: $BASE_URL/swagger-ui/"
echo ""

# Step 3: Run API tests
echo -e "${YELLOW}Step 3: Running API Tests${NC}"

echo -e "\n${YELLOW}3.1. Testing health check...${NC}"
if curl -s -f "$BASE_URL/health" > /dev/null; then
    echo -e "${GREEN}✓ Health check passed${NC}"
else
    echo -e "${RED}✗ Health check failed${NC}"
    echo -e "${RED}❌ Server may not be running at $BASE_URL${NC}"
    echo "Please start the server first: ./scripts/run-local.sh"
    exit 1
fi

echo -e "\n${YELLOW}3.2. Testing OpenAPI documentation...${NC}"
if curl -s -f "$BASE_URL/api-docs/openapi.json" > /dev/null; then
    echo -e "${GREEN}✓ OpenAPI documentation available${NC}"
    echo "   View API docs at: $BASE_URL/swagger-ui/"
else
    echo -e "${RED}✗ OpenAPI documentation failed${NC}"
fi

# Test the debug endpoint
echo -e "\n${YELLOW}3.3. Testing transaction debug...${NC}"
curl -X POST "$BASE_URL/debug" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d "{
    \"signature\": \"$SIGNATURE\",
    \"rpc_url\": \"$RPC_URL\"
  }" \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n" \
  | jq '.' 2>/dev/null || echo "Response received (jq not available for pretty printing)"

echo ""
echo -e "${GREEN}✓ API test completed!${NC}"
