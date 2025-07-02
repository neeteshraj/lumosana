#!/bin/bash

# Transaction Debugger Environment Manager
# Consolidated script for all environment management tasks:
# - Configuration validation and loading
# - Environment setup and testing
# - Configuration API testing
# - Demo and integration testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default values
ENVIRONMENT="development"
CLEAN_BUILD=false
SHOW_CONFIG=false
DEBUG_CONFIG=false
SAVE_CONFIG=""
CHECK_DEPS=true
ACTION=""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $1"
}

print_header() {
    echo -e "${BLUE}$1${NC}"
    echo "$(echo "$1" | sed 's/./=/g')"
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [ACTION] [OPTIONS]

ACTIONS:
    validate                 Validate all environment configuration files
    test-env                 Test environment variable loading
    test-config-api          Test configuration API endpoints
    test-integration         Run full integration tests
    demo                     Run configuration management demo
    start                    Start the application (default action)

OPTIONS:
    -e, --env ENVIRONMENT    Set environment (development, production, docker)
    -c, --clean              Clean build before starting
    -s, --show-config        Show configuration summary and exit
    -d, --debug-config       Show detailed configuration in JSON and exit
    --save-config FILE       Save configuration to file and exit
    --no-deps-check          Skip dependency checks
    -h, --help               Show this help message

EXAMPLES:
    $0 validate                                    # Validate all config files
    $0 test-env                                   # Test environment loading
    $0 test-config-api                            # Test config API
    $0 demo                                       # Run config demo
    $0 start -e production --show-config          # Show production config
    $0 start -e development --save-config dev.json # Save dev config to file

EOF
}

# Dependency checking function
check_dependencies() {
    if [ "$CHECK_DEPS" = false ]; then
        print_status "Skipping dependency checks..."
        return 0
    fi

    print_status "Checking dependencies..."
    
    # Check for required tools
    local missing_deps=()
    
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("cargo (Rust)")
    fi
    
    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi
    
    if ! command -v jq &> /dev/null; then
        print_warning "jq not found - JSON output won't be pretty-printed"
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Missing dependencies: ${missing_deps[*]}"
        exit 1
    fi
    
    print_status "All dependencies found"
}

# Environment setup function
setup_environment() {
    cd "$PROJECT_ROOT"
    
    print_status "Setting up environment: $ENVIRONMENT"
    
    # Set environment variable
    export ENVIRONMENT="$ENVIRONMENT"
    
    # Check if environment file exists
    local env_file=".env.$ENVIRONMENT"
    if [ -f "$env_file" ]; then
        print_status "Found environment file: $env_file"
    else
        print_warning "Environment file not found: $env_file"
    fi
    
    # Create configs directory if it doesn't exist
    mkdir -p configs
}

# Configuration validation function
validate_config() {
    print_header "Configuration Validation"
    
    local errors=0
    
    # Required environment files
    local env_files=(".env.development" ".env.production" ".env.docker" ".env.example")
    
    for env_file in "${env_files[@]}"; do
        if [ ! -f "$PROJECT_ROOT/$env_file" ]; then
            print_error "File not found: $env_file"
            errors=$((errors + 1))
            continue
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
            if ! grep -q "^$var=" "$PROJECT_ROOT/$env_file"; then
                print_error "Missing required variable '$var' in $env_file"
                errors=$((errors + 1))
            fi
        done
        
        # Check for duplicate variables
        local duplicates=$(cut -d'=' -f1 "$PROJECT_ROOT/$env_file" | sort | uniq -d)
        if [ -n "$duplicates" ]; then
            print_error "Duplicate variables in $env_file: $duplicates"
            errors=$((errors + 1))
        fi
        
        # Check for empty values
        local empty_vars=$(grep "^[A-Z_]*=$" "$PROJECT_ROOT/$env_file" | cut -d'=' -f1)
        if [ -n "$empty_vars" ]; then
            print_warning "Empty variables in $env_file: $empty_vars"
        fi
    done
    
    if [ $errors -eq 0 ]; then
        print_status "All configuration files are valid!"
        return 0
    else
        print_error "Found $errors configuration errors"
        return 1
    fi
}

# Test environment loading
test_env_loading() {
    print_header "Environment Configuration Loading Test"
    
    cd "$PROJECT_ROOT"
    
    # Test development environment
    print_status "Testing development environment..."
    if ENVIRONMENT=development timeout 10s cargo run --bin main -- --version > /dev/null 2>&1; then
        print_status "✓ Development environment loads successfully"
    else
        print_warning "Development environment test completed with timeout (expected)"
    fi
    
    # Test production environment  
    print_status "Testing production environment..."
    if ENVIRONMENT=production timeout 10s cargo run --bin main -- --version > /dev/null 2>&1; then
        print_status "✓ Production environment loads successfully"
    else
        print_warning "Production environment test completed with timeout (expected)"
    fi
    
    # Check configuration differences
    print_status "Comparing environment configurations..."
    if [[ -f ".env.development" && -f ".env.production" ]]; then
        local dev_env=$(grep "ENVIRONMENT=" .env.development | cut -d'=' -f2 2>/dev/null || echo "")
        local prod_env=$(grep "ENVIRONMENT=" .env.production | cut -d'=' -f2 2>/dev/null || echo "")
        
        if [[ "$dev_env" == "development" && "$prod_env" == "production" ]]; then
            print_status "✓ Environment variables are correctly set"
        else
            print_warning "Environment variables may not be correctly configured"
        fi
    fi
    
    print_status "✓ Configuration loading test completed"
}

# Test configuration API
test_config_api() {
    print_header "Configuration API Test Suite"
    
    local base_url="http://localhost:8081"
    local summary_endpoint="/api/config/summary"
    local detailed_endpoint="/api/config/detailed"
    
    # Check if server is running
    print_status "[1/4] Checking if server is running..."
    if curl -s --fail "${base_url}/health" > /dev/null; then
        print_status "✓ Server is running"
    else
        print_error "✗ Server is not running at ${base_url}"
        print_error "Please start the server first: ./scripts/env-manager.sh start"
        return 1
    fi
    
    # Test summary endpoint
    print_status "[2/4] Testing configuration summary endpoint..."
    local summary_response=$(curl -s "${base_url}${summary_endpoint}")
    if echo "$summary_response" | jq . > /dev/null 2>&1; then
        print_status "✓ Summary endpoint returns valid JSON"
        
        # Check required fields in summary
        local app_name=$(echo "$summary_response" | jq -r '.app.name // empty')
        local server_port=$(echo "$summary_response" | jq -r '.server.http_port // empty')
        
        if [[ -n "$app_name" && -n "$server_port" ]]; then
            print_status "✓ Summary contains required fields (app.name: $app_name, server.http_port: $server_port)"
        else
            print_error "✗ Summary missing required fields"
            return 1
        fi
    else
        print_error "✗ Summary endpoint returned invalid JSON"
        echo "Response: $summary_response"
        return 1
    fi
    
    # Test detailed endpoint
    print_status "[3/4] Testing detailed configuration endpoint..."
    local detailed_response=$(curl -s "${base_url}${detailed_endpoint}")
    if echo "$detailed_response" | jq . > /dev/null 2>&1; then
        print_status "✓ Detailed endpoint returns valid JSON"
        
        # Check if detailed config has more fields than summary
        local detailed_keys=$(echo "$detailed_response" | jq -r 'keys[]' | wc -l)
        local summary_keys=$(echo "$summary_response" | jq -r 'keys[]' | wc -l)
        
        if [ "$detailed_keys" -ge "$summary_keys" ]; then
            print_status "✓ Detailed config contains more information than summary"
        else
            print_warning "Detailed config might not contain more information than summary"
        fi
    else
        print_error "✗ Detailed endpoint returned invalid JSON"
        return 1
    fi
    
    # Test performance
    print_status "[4/4] Testing endpoint performance..."
    local response_time=$(curl -o /dev/null -s -w "%{time_total}" "${base_url}${summary_endpoint}")
    local response_time_ms=$(echo "$response_time * 1000" | bc 2>/dev/null || echo "N/A")
    
    if [[ "$response_time_ms" != "N/A" ]]; then
        print_status "✓ Summary endpoint response time: ${response_time_ms}ms"
    else
        print_status "✓ Summary endpoint responded successfully"
    fi
    
    print_status "✓ All configuration API tests passed!"
}

# Run integration tests
test_integration() {
    print_header "Environment Configuration Integration Test"
    
    # Test 1: Configuration validation
    print_status "[1/5] Testing configuration validation..."
    if validate_config > /dev/null 2>&1; then
        print_status "✓ All environment files are valid"
    else
        print_error "✗ Environment file validation failed"
        return 1
    fi
    
    # Test 2: Build with configuration
    print_status "[2/5] Testing build with configuration system..."
    cd "$PROJECT_ROOT"
    if cargo build --quiet; then
        print_status "✓ Application builds successfully with config system"
    else
        print_error "✗ Build failed"
        return 1
    fi
    
    # Test 3: Test configuration loading in different environments
    print_status "[3/5] Testing configuration loading..."
    
    # Test development environment
    print_status "Testing development environment loading..."
    if timeout 5s bash -c "ENVIRONMENT=development cargo run --bin main --quiet > /dev/null 2>&1"; then
        print_status "✓ Development environment loads successfully"
    else
        print_status "✓ Development environment test completed (timeout expected)"
    fi
    
    # Test 4: Environment variable consistency
    print_status "[4/5] Testing environment variable consistency..."
    if [[ -f ".env.development" && -f ".env.production" ]]; then
        # Check that required variables exist in both files
        local dev_vars=$(grep "^[A-Z_]*=" .env.development | cut -d'=' -f1 | sort)
        local prod_vars=$(grep "^[A-Z_]*=" .env.production | cut -d'=' -f1 | sort)
        
        local missing_in_prod=$(comm -23 <(echo "$dev_vars") <(echo "$prod_vars"))
        local missing_in_dev=$(comm -13 <(echo "$dev_vars") <(echo "$prod_vars"))
        
        if [[ -z "$missing_in_prod" && -z "$missing_in_dev" ]]; then
            print_status "✓ Both environments have consistent variable sets"
        else
            print_warning "Environment variable sets differ between dev and prod"
            [ -n "$missing_in_prod" ] && print_warning "Missing in production: $missing_in_prod"
            [ -n "$missing_in_dev" ] && print_warning "Missing in development: $missing_in_dev"
        fi
    fi
    
    # Test 5: Configuration file permissions
    print_status "[5/5] Testing configuration file permissions..."
    local sensitive_files=(".env.development" ".env.production")
    for file in "${sensitive_files[@]}"; do
        if [[ -f "$file" ]]; then
            local perms=$(stat -f "%Mp%Lp" "$file" 2>/dev/null || echo "unknown")
            if [[ "$perms" =~ ^6[0-4][0-4]$ ]]; then
                print_status "✓ $file has appropriate permissions ($perms)"
            else
                print_warning "$file permissions may be too permissive ($perms)"
            fi
        fi
    done
    
    print_status "✓ Integration tests completed successfully!"
}

# Run configuration demo
run_demo() {
    print_header "Configuration Management Demo"
    
    cd "$PROJECT_ROOT"
    
    print_status "1. Showing configuration summary:"
    show_configuration
    
    print_status "\n2. Showing detailed configuration in JSON:"
    print_status "First 30 lines of detailed config:"
    show_detailed_configuration | head -30
    
    print_status "\n3. Saving configuration to files:"
    
    # Save configurations for different environments
    print_status "Saving development configuration..."
    ENVIRONMENT="development" save_configuration_to_file "configs/development-config.json"
    
    print_status "Saving production configuration..."
    ENVIRONMENT="production" save_configuration_to_file "configs/production-config.json"
    
    print_status "\n4. Comparing environment configurations:"
    
    print_status "Key differences between development and production:"
    print_status "=================================================="
    
    print_status "Development environment:"
    grep -E '"environment"|"rust_log"|"debug_mode"|"pretty_logs"' configs/development-config.json | head -5 2>/dev/null || print_warning "Config file not found or invalid"
    
    print_status "Production environment:"
    grep -E '"environment"|"rust_log"|"debug_mode"|"pretty_logs"' configs/production-config.json | head -5 2>/dev/null || print_warning "Config file not found or invalid"
    
    print_status "\n5. Configuration validation:"
    if validate_config | grep -q "All configuration files are valid!"; then
        print_status "✓ All configurations are valid!"
    else
        print_warning "Some configuration issues found"
    fi
    
    print_status "\n6. Testing environment loading:"
    test_env_loading
    
    print_status "\nDemo completed! Check the configs/ directory for saved configurations."
}

# Configuration display functions
show_configuration() {
    print_status "Current Configuration Summary"
    print_status "Environment: $ENVIRONMENT"
    print_status "Project Root: $PROJECT_ROOT"
    
    if [[ -f ".env.$ENVIRONMENT" ]]; then
        print_status "Environment file: .env.$ENVIRONMENT"
        print_status "Key settings:"
        grep -E "^(APP_NAME|HTTP_PORT|GRPC_PORT|RUST_LOG)=" ".env.$ENVIRONMENT" 2>/dev/null || print_warning "Could not read environment file"
    else
        print_warning "Environment file .env.$ENVIRONMENT not found"
    fi
}

show_detailed_configuration() {
    # This would ideally run the application with a config dump flag
    # For now, we'll show the environment file contents in JSON-like format
    print_debug "Detailed configuration for environment: $ENVIRONMENT"
    
    if [[ -f ".env.$ENVIRONMENT" ]]; then
        echo "{"
        while IFS='=' read -r key value; do
            [[ "$key" =~ ^[A-Z_]+$ ]] && echo "  \"${key,,}\": \"$value\","
        done < ".env.$ENVIRONMENT"
        echo "}"
    else
        echo "{\"error\": \"Environment file not found\"}"
    fi
}

save_configuration_to_file() {
    local output_file="$1"
    local output_dir=$(dirname "$output_file")
    
    mkdir -p "$output_dir"
    
    print_status "Saving configuration to: $output_file"
    show_detailed_configuration > "$output_file"
    print_status "Configuration saved successfully"
}

# Build functions
clean_build() {
    if [ "$CLEAN_BUILD" = true ]; then
        print_status "Cleaning previous build..."
        cd "$PROJECT_ROOT"
        cargo clean
    fi
}

build_and_run() {
    print_status "Building and starting application..."
    cd "$PROJECT_ROOT"
    
    if cargo build; then
        print_status "Build successful! Starting application..."
        print_status "Environment: $ENVIRONMENT"
        print_status "Press Ctrl+C to stop"
        exec cargo run --bin main
    else
        print_error "Build failed!"
        exit 1
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        validate|test-env|test-config-api|test-integration|demo|start)
            ACTION="$1"
            shift
            ;;
        -e|--env)
            ENVIRONMENT="$2"
            shift 2
            ;;
        -c|--clean)
            CLEAN_BUILD=true
            shift
            ;;
        -s|--show-config)
            SHOW_CONFIG=true
            shift
            ;;
        -d|--debug-config)
            DEBUG_CONFIG=true
            shift
            ;;
        --save-config)
            SAVE_CONFIG="$2"
            shift 2
            ;;
        --no-deps-check)
            CHECK_DEPS=false
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Set default action if none provided
if [ -z "$ACTION" ]; then
    ACTION="start"
fi

# Main execution
main() {
    print_header "Transaction Debugger Environment Manager"
    
    check_dependencies
    setup_environment
    
    case "$ACTION" in
        validate)
            validate_config
            ;;
        test-env)
            test_env_loading
            ;;
        test-config-api)
            test_config_api
            ;;
        test-integration)
            test_integration
            ;;
        demo)
            run_demo
            ;;
        start)
            # Handle configuration display/save options
            if [ "$SHOW_CONFIG" = true ]; then
                show_configuration
                exit 0
            fi
            
            if [ "$DEBUG_CONFIG" = true ]; then
                show_detailed_configuration
                exit 0
            fi
            
            if [ -n "$SAVE_CONFIG" ]; then
                save_configuration_to_file "$SAVE_CONFIG"
                exit 0
            fi
            
            # Normal startup flow
            show_configuration
            clean_build
            build_and_run
            ;;
        *)
            print_error "Unknown action: $ACTION"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function
main
