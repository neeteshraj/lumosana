#!/bin/bash

# Configuration API Test Script
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BASE_URL="http://localhost:8081"
SUMMARY_ENDPOINT="/api/config/summary"
DETAILED_ENDPOINT="/api/config/detailed"

echo -e "${BLUE}Configuration API Test Suite${NC}"
echo "=================================="

# Check if server is running
echo -e "\n${YELLOW}[1/4] Checking if server is running...${NC}"
if curl -s --fail "${BASE_URL}/health" > /dev/null; then
    echo -e "${GREEN}✓ Server is running${NC}"
else
    echo -e "${RED}✗ Server is not running at ${BASE_URL}${NC}"
    echo "Please start the server first: ./scripts/start.sh"
    exit 1
fi

# Test summary endpoint
echo -e "\n${YELLOW}[2/4] Testing configuration summary endpoint...${NC}"
SUMMARY_RESPONSE=$(curl -s "${BASE_URL}${SUMMARY_ENDPOINT}")
if echo "$SUMMARY_RESPONSE" | jq . > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Summary endpoint returns valid JSON${NC}"
    
    # Check required fields in summary
    APP_NAME=$(echo "$SUMMARY_RESPONSE" | jq -r '.app.name // empty')
    SERVER_PORT=$(echo "$SUMMARY_RESPONSE" | jq -r '.server.http_port // empty')
    
    if [[ -n "$APP_NAME" && -n "$SERVER_PORT" ]]; then
        echo -e "${GREEN}✓ Summary contains required fields (app.name: $APP_NAME, server.http_port: $SERVER_PORT)${NC}"
    else
        echo -e "${RED}✗ Summary missing required fields${NC}"
        exit 1
    fi
else
    echo -e "${RED}✗ Summary endpoint returned invalid JSON${NC}"
    echo "Response: $SUMMARY_RESPONSE"
    exit 1
fi

# Test detailed endpoint
echo -e "\n${YELLOW}[3/4] Testing configuration detailed endpoint...${NC}"
DETAILED_RESPONSE=$(curl -s "${BASE_URL}${DETAILED_ENDPOINT}")
if echo "$DETAILED_RESPONSE" | jq . > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Detailed endpoint returns valid JSON${NC}"
    
    # Check if detailed has more fields than summary
    DETAILED_KEYS=$(echo "$DETAILED_RESPONSE" | jq 'keys | length')
    SUMMARY_KEYS=$(echo "$SUMMARY_RESPONSE" | jq 'keys | length')
    
    if [[ "$DETAILED_KEYS" -ge "$SUMMARY_KEYS" ]]; then
        echo -e "${GREEN}✓ Detailed endpoint contains more or equal configuration sections ($DETAILED_KEYS >= $SUMMARY_KEYS)${NC}"
    else
        echo -e "${RED}✗ Detailed endpoint has fewer sections than summary${NC}"
        exit 1
    fi
else
    echo -e "${RED}✗ Detailed endpoint returned invalid JSON${NC}"
    echo "Response: $DETAILED_RESPONSE"
    exit 1
fi

# Test response structure
echo -e "\n${YELLOW}[4/4] Validating response structure...${NC}"

# Required sections in detailed config
REQUIRED_SECTIONS=("app" "server" "database" "observability" "security" "health" "development")
MISSING_SECTIONS=()

for section in "${REQUIRED_SECTIONS[@]}"; do
    if echo "$DETAILED_RESPONSE" | jq -e ".$section" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Section '$section' present${NC}"
    else
        MISSING_SECTIONS+=("$section")
    fi
done

if [[ ${#MISSING_SECTIONS[@]} -eq 0 ]]; then
    echo -e "\n${GREEN}✓ All required configuration sections present${NC}"
else
    echo -e "\n${RED}✗ Missing sections: ${MISSING_SECTIONS[*]}${NC}"
    exit 1
fi

# Show sample configuration for verification
echo -e "\n${BLUE}Configuration Summary:${NC}"
echo "======================"
echo "$SUMMARY_RESPONSE" | jq .

echo -e "\n${GREEN}✓ All configuration API tests passed!${NC}"
echo -e "${BLUE}Configuration endpoints are working correctly.${NC}"
