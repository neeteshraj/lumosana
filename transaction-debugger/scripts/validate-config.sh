#!/bin/bash

# Configuration Validator Script
# Validates environment configuration files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to validate environment file
validate_env_file() {
    local env_file="$1"
    local errors=0
    
    if [ ! -f "$env_file" ]; then
        print_error "File not found: $env_file"
        return 1
    fi
    
    print_status "Validating $env_file..."
    
    # Required variables
    local required_vars=(
        "APP_NAME"
        "HTTP_PORT" 
        "GRPC_PORT"
        "RUST_LOG"
        "OTEL_SERVICE_NAME"
    )
    
    # Check required variables
    for var in "${required_vars[@]}"; do
        if ! grep -q "^$var=" "$env_file"; then
            print_error "Missing required variable: $var"
            ((errors++))
        fi
    done
    
    # Validate port numbers
    local ports=("HTTP_PORT" "GRPC_PORT" "JAEGER_AGENT_PORT" "METRICS_PORT")
    for port_var in "${ports[@]}"; do
        local port_value=$(grep "^$port_var=" "$env_file" 2>/dev/null | cut -d'=' -f2)
        if [ -n "$port_value" ]; then
            if ! [[ "$port_value" =~ ^[0-9]+$ ]] || [ "$port_value" -lt 1 ] || [ "$port_value" -gt 65535 ]; then
                print_error "Invalid port number for $port_var: $port_value"
                ((errors++))
            fi
        fi
    done
    
    # Validate boolean values
    local booleans=("ENABLE_METRICS" "ENABLE_TRACING" "ENABLE_LOGGING" "DEBUG_MODE" "HOT_RELOAD" "PRETTY_LOGS")
    for bool_var in "${booleans[@]}"; do
        local bool_value=$(grep "^$bool_var=" "$env_file" 2>/dev/null | cut -d'=' -f2)
        if [ -n "$bool_value" ]; then
            if ! [[ "$bool_value" =~ ^(true|false)$ ]]; then
                print_error "Invalid boolean value for $bool_var: $bool_value (must be 'true' or 'false')"
                ((errors++))
            fi
        fi
    done
    
    # Validate trace sampling rate
    local sampling_rate=$(grep "^TRACE_SAMPLING_RATE=" "$env_file" 2>/dev/null | cut -d'=' -f2)
    if [ -n "$sampling_rate" ]; then
        if ! [[ "$sampling_rate" =~ ^[0-9]*\.?[0-9]+$ ]] || (( $(echo "$sampling_rate < 0.0" | bc -l) )) || (( $(echo "$sampling_rate > 1.0" | bc -l) )); then
            print_error "Invalid trace sampling rate: $sampling_rate (must be between 0.0 and 1.0)"
            ((errors++))
        fi
    fi
    
    # Validate URLs
    local urls=("SOLANA_RPC_URL" "SOLANA_WEBSOCKET_URL" "JAEGER_COLLECTOR_ENDPOINT" "OTEL_EXPORTER_JAEGER_ENDPOINT")
    for url_var in "${urls[@]}"; do
        local url_value=$(grep "^$url_var=" "$env_file" 2>/dev/null | cut -d'=' -f2)
        if [ -n "$url_value" ]; then
            if ! [[ "$url_value" =~ ^https?:// ]] && ! [[ "$url_value" =~ ^wss?:// ]]; then
                print_warning "URL for $url_var might be invalid: $url_value"
            fi
        fi
    done
    
    if [ $errors -eq 0 ]; then
        print_status "✓ $env_file is valid"
        return 0
    else
        print_error "✗ $env_file has $errors error(s)"
        return 1
    fi
}

# Main function
main() {
    print_status "Configuration Validator"
    print_status "======================="
    
    cd "$PROJECT_ROOT"
    
    local total_errors=0
    
    # Validate all environment files
    for env_file in .env .env.example .env.development .env.production; do
        if [ -f "$env_file" ]; then
            if ! validate_env_file "$env_file"; then
                ((total_errors++))
            fi
            echo
        else
            print_warning "Environment file not found: $env_file"
        fi
    done
    
    if [ $total_errors -eq 0 ]; then
        print_status "All configuration files are valid!"
        exit 0
    else
        print_error "Found errors in $total_errors configuration file(s)"
        exit 1
    fi
}

# Check if bc is available for floating point comparisons
if ! command -v bc &> /dev/null; then
    print_warning "bc not found, skipping trace sampling rate validation"
fi

main
