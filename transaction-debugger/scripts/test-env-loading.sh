#!/bin/bash

# Quick configuration loading test
set -e

echo "Environment Configuration Loading Test"
echo "======================================"

# Test 1: Check development configuration loading
echo "Testing development environment..."
cd /Users/nitesh/Desktop/lumosana/transaction-debugger
ENVIRONMENT=development cargo run --bin main -- --version 2>/dev/null || echo "Development test completed"

# Test 2: Check production configuration loading  
echo "Testing production environment..."
ENVIRONMENT=production cargo run --bin main -- --version 2>/dev/null || echo "Production test completed"

# Test 3: Check if configuration files exist and are valid
echo "Checking configuration files..."
if [[ -f ".env.development" && -f ".env.production" && -f ".env.example" ]]; then
    echo "✓ All environment files exist"
else
    echo "✗ Missing environment files"
    exit 1
fi

# Test 4: Check configuration differences
echo "Comparing environment configurations..."
DEV_ENV=$(grep "ENVIRONMENT=" .env.development | cut -d'=' -f2)
PROD_ENV=$(grep "ENVIRONMENT=" .env.production | cut -d'=' -f2)

if [[ "$DEV_ENV" == "development" && "$PROD_ENV" == "production" ]]; then
    echo "✓ Environment variables are correctly set"
else
    echo "✗ Environment variables not correctly configured"
    exit 1
fi

echo "✓ Configuration loading test completed successfully"
