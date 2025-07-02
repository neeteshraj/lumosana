#!/bin/bash

# Configuration Management Demo Script
# This script demonstrates all the configuration features

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Configuration Management Demo${NC}"
echo "=================================="

echo -e "\n${YELLOW}1. Showing configuration summary:${NC}"
./scripts/start.sh --show-config

echo -e "\n${YELLOW}2. Showing detailed configuration in JSON:${NC}"
echo "First 30 lines of detailed config:"
./scripts/start.sh --debug-config | head -30

echo -e "\n${YELLOW}3. Saving configuration to files:${NC}"

# Save configurations for different environments
echo "Saving development configuration..."
./scripts/start.sh -e development --save-config configs/development-config.json

echo "Saving production configuration..."
./scripts/start.sh -e production --save-config configs/production-config.json

echo -e "\n${YELLOW}4. Comparing environment configurations:${NC}"

# Create configs directory if it doesn't exist
mkdir -p configs

# Compare key differences between environments
echo "Key differences between development and production:"
echo "=================================================="

echo -e "${GREEN}Development environment:${NC}"
grep -E '"environment"|"rust_log"|"debug_mode"|"pretty_logs"' configs/development-config.json | head -5

echo -e "${GREEN}Production environment:${NC}"
grep -E '"environment"|"rust_log"|"debug_mode"|"pretty_logs"' configs/production-config.json | head -5

echo -e "\n${YELLOW}5. Configuration validation:${NC}"
if ./scripts/validate-config.sh | grep -q "All configuration files are valid!"; then
    echo -e "${GREEN}✓ All configuration files are valid${NC}"
else
    echo -e "${RED}✗ Configuration validation failed${NC}"
fi

echo -e "\n${YELLOW}6. Available configuration files:${NC}"
ls -la configs/*.json 2>/dev/null || echo "No saved configuration files found"

echo -e "\n${BLUE}Configuration Management Demo Complete!${NC}"
echo "======================================="

echo -e "\n${GREEN}Available commands:${NC}"
echo "  ./scripts/start.sh --show-config              # Show summary"
echo "  ./scripts/start.sh --debug-config             # Show detailed JSON"
echo "  ./scripts/start.sh --save-config <file>       # Save to file"
echo "  ./scripts/start.sh -e production --debug-config  # Show prod config"
echo "  ./scripts/validate-config.sh                  # Validate all env files"
echo "  ./scripts/test-config-api.sh                  # Test config API endpoints"
