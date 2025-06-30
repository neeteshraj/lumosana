#!/bin/bash

if [ -z "$1" ]; then
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

echo "Testing Transaction Debugger API..."
echo "Signature: $SIGNATURE"
echo "RPC URL: $RPC_URL"
echo "API Base URL: $BASE_URL"
echo "Swagger UI: $BASE_URL/swagger-ui/"
echo ""

echo "1. Testing health check..."
curl -f "$BASE_URL/health" && echo "✓ Health check passed" || echo "✗ Health check failed"
echo ""

echo "2. Testing OpenAPI documentation..."
curl -f "$BASE_URL/api-docs/openapi.json" > /dev/null && echo "✓ OpenAPI documentation available" || echo "✗ OpenAPI documentation failed"
echo "   View API docs at: $BASE_URL/swagger-ui/"
echo ""

# Test the debug endpoint
echo "3. Testing transaction debug..."
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
echo "Test completed!"
