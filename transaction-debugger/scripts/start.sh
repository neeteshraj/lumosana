#!/bin/bash

# Transaction Debugger Startup Script
# This script helps manage different environment configurations

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

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Options:
    -e, --env ENVIRONMENT    Set environment (development, production, docker)
    -c, --clean             Clean build before starting
    -s, --show-config       Show configuration summary
    -d, --debug-config      Show detailed configuration in JSON format
    --save-config FILE      Save detailed configuration to file
    -h, --help              Show this help message
    --no-deps-check         Skip dependency checks

Environments:
    development             Use .env.development configuration
    production              Use .env.production configuration  
    docker                  Use Docker environment configuration

Examples:
    $0                      # Start with development environment
    $0 -e production        # Start with production environment
    $0 -c -e development    # Clean build and start with development
    $0 --show-config        # Show config summary and exit
    $0 --debug-config       # Show detailed config in JSON and exit
    $0 --save-config config.json  # Save config to file and exit

EOF
}

# Function to check dependencies
check_dependencies() {
    if [ "$CHECK_DEPS" = false ]; then
        return 0
    fi

    print_status "Checking dependencies..."
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo (Rust) is not installed. Please install Rust first."
        exit 1
    fi
    
    # Check if required environment file exists
    local env_file="$PROJECT_ROOT/.env.$ENVIRONMENT"
    if [ "$ENVIRONMENT" != "docker" ] && [ ! -f "$env_file" ]; then
        print_warning "Environment file $env_file not found."
        print_status "Creating from .env.example..."
        
        if [ -f "$PROJECT_ROOT/.env.example" ]; then
            cp "$PROJECT_ROOT/.env.example" "$env_file"
            print_status "Created $env_file from .env.example"
        else
            print_error ".env.example not found. Cannot create environment file."
            exit 1
        fi
    fi
}

# Function to setup environment
setup_environment() {
    print_status "Setting up environment: $ENVIRONMENT"
    
    cd "$PROJECT_ROOT"
    
    case "$ENVIRONMENT" in
        "development")
            if [ -f ".env.development" ]; then
                cp .env.development .env
                print_status "Using development configuration"
            else
                print_warning "No .env.development found, using default .env"
            fi
            ;;
        "production")
            if [ -f ".env.production" ]; then
                cp .env.production .env
                print_status "Using production configuration"
            else
                print_error "No .env.production found"
                exit 1
            fi
            ;;
        "docker")
            print_status "Using Docker environment configuration"
            # Docker will handle environment variables
            ;;
        *)
            print_error "Unknown environment: $ENVIRONMENT"
            exit 1
            ;;
    esac
}

# Function to show configuration
show_configuration() {
    print_status "Current Configuration:"
    echo "  Environment: $ENVIRONMENT"
    echo "  Project Root: $PROJECT_ROOT"
    
    if [ -f "$PROJECT_ROOT/.env" ]; then
        echo "  Configuration file: .env"
        echo ""
        echo "Key configuration values:"
        grep -E "^(APP_NAME|ENVIRONMENT|HTTP_PORT|GRPC_PORT|RUST_LOG|OTEL_SERVICE_NAME)=" "$PROJECT_ROOT/.env" | sed 's/^/    /'
    else
        echo "  No .env file found"
    fi
    echo ""
}

# Function to show detailed configuration using Rust
show_detailed_configuration() {
    print_status "Showing detailed configuration in JSON format..."
    cd "$PROJECT_ROOT"
    
    # Create a temporary Rust binary to print detailed config
    cat > /tmp/print_config.rs << 'EOF'
use transaction_debugger::config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    config.print_detailed();
    Ok(())
}
EOF

    # Compile and run the config printer
    if cargo run --bin main --quiet -- --print-detailed 2>/dev/null; then
        echo "Detailed configuration printed above"
    else
        print_warning "Failed to load detailed configuration, falling back to basic config"
        show_configuration
    fi
}

# Function to save configuration to file using Rust
save_configuration_to_file() {
    local file_path="$1"
    print_status "Saving detailed configuration to: $file_path"
    cd "$PROJECT_ROOT"
    
    # Create a temporary Rust binary to save config
    cat > /tmp/save_config.rs << 'EOF'
use transaction_debugger::config::Config;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        config.save_to_file(&args[1])?;
    }
    Ok(())
}
EOF

    # Try to save using our Rust config
    if cargo run --bin main --quiet -- --save-config "$file_path" 2>/dev/null; then
        print_status "Configuration saved successfully"
    else
        print_warning "Failed to save using Rust config, creating basic JSON file"
        # Fallback: create a basic config file
        cat > "$file_path" << EOF
{
  "note": "Basic configuration export",
  "environment": "$ENVIRONMENT",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "env_file_content": $(if [ -f .env ]; then jq -R -s 'split("\n") | map(select(length > 0 and startswith("#") | not)) | map(split("=") | {key: .[0], value: (.[1:] | join("="))}) | from_entries' .env 2>/dev/null || echo '{}'; else echo '{}'; fi)
}
EOF
        print_status "Basic configuration saved to: $file_path"
    fi
}

# Function to clean build
clean_build() {
    if [ "$CLEAN_BUILD" = true ]; then
        print_status "Cleaning previous build..."
        cd "$PROJECT_ROOT"
        cargo clean
    fi
}

# Function to build and run
build_and_run() {
    print_status "Building and running Transaction Debugger..."
    cd "$PROJECT_ROOT"
    
    if [ "$ENVIRONMENT" = "docker" ]; then
        print_status "Starting with Docker Compose..."
        cd infrastructure/docker
        docker-compose up --build
    else
        print_status "Building Rust application..."
        cargo build --release
        
        print_status "Starting application..."
        cargo run --release
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
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

# Main execution
main() {
    print_status "Transaction Debugger Startup Script"
    print_status "===================================="
    
    check_dependencies
    setup_environment
    
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
}

# Run main function
main
