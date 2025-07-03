#!/bin/bash

# Lumosana Kubernetes Deployment Script
# Deploys the complete Lumosana observability stack to Kubernetes

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
NAMESPACE="lumosana"
CONTEXT=""
DRY_RUN=false
SKIP_BUILD=false

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Show usage
show_usage() {
    cat << EOF
Usage: $0 [options]

Deploy Lumosana with full observability stack to Kubernetes

Options:
    -h, --help              Show this help message
    -n, --namespace NAME    Kubernetes namespace (default: lumosana)
    -c, --context CONTEXT   Kubernetes context to use
    -d, --dry-run          Perform a dry run without applying changes
    -s, --skip-build       Skip Docker image build step
    --delete               Delete the deployment instead of creating it

Examples:
    $0                      # Deploy to 'lumosana' namespace
    $0 -n production        # Deploy to 'production' namespace
    $0 --dry-run           # See what would be deployed
    $0 --delete            # Delete the deployment

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -n|--namespace)
            NAMESPACE="$2"
            shift 2
            ;;
        -c|--context)
            CONTEXT="$2"
            shift 2
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -s|--skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --delete)
            DELETE_MODE=true
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    if ! command -v kubectl &> /dev/null; then
        print_error "kubectl is not installed or not in PATH"
        exit 1
    fi
    
    if ! command -v docker &> /dev/null && [ "$SKIP_BUILD" = false ]; then
        print_error "Docker is not installed or not in PATH"
        exit 1
    fi
    
    # Check kubectl connection
    if ! kubectl cluster-info &> /dev/null; then
        print_error "Cannot connect to Kubernetes cluster"
        print_error "Please check your kubeconfig and cluster connectivity"
        exit 1
    fi
    
    print_success "Prerequisites check passed"
}

# Set kubectl context if specified
set_context() {
    if [ -n "$CONTEXT" ]; then
        print_status "Setting kubectl context to: $CONTEXT"
        kubectl config use-context "$CONTEXT"
    fi
    
    local current_context=$(kubectl config current-context)
    print_status "Using kubectl context: $current_context"
}

# Build Docker image
build_image() {
    if [ "$SKIP_BUILD" = true ]; then
        print_warning "Skipping Docker image build"
        return
    fi
    
    print_status "Building Docker image..."
    cd "$(dirname "$0")/../../transaction-debugger"
    
    if [ "$DRY_RUN" = false ]; then
        docker build -t lumosana/transaction-debugger:latest .
        print_success "Docker image built successfully"
    else
        print_status "DRY RUN: Would build Docker image lumosana/transaction-debugger:latest"
    fi
    
    cd - > /dev/null
}

# Create or ensure namespace exists
setup_namespace() {
    print_status "Setting up namespace: $NAMESPACE"
    
    if [ "$DRY_RUN" = false ]; then
        kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
        print_success "Namespace '$NAMESPACE' ready"
    else
        print_status "DRY RUN: Would create/ensure namespace '$NAMESPACE'"
    fi
}

# Apply Kubernetes manifests
apply_manifests() {
    local k8s_dir="$(dirname "$0")/../k8s"
    
    print_status "Applying Kubernetes manifests..."
    
    # Define deployment order for dependencies
    local manifests=(
        "configmap.yaml"
        "deployment.yaml"
        "service.yaml"
        "envoy.yaml"
    )
    
    for manifest in "${manifests[@]}"; do
        local file_path="$k8s_dir/$manifest"
        
        if [ -f "$file_path" ]; then
            print_status "Applying $manifest..."
            
            if [ "$DRY_RUN" = false ]; then
                kubectl apply -f "$file_path" -n "$NAMESPACE"
                print_success "Applied $manifest"
            else
                print_status "DRY RUN: Would apply $manifest"
            fi
        else
            print_warning "Manifest file not found: $file_path"
        fi
    done
    
    print_success "All manifests applied successfully"
}

# Delete deployment
delete_deployment() {
    print_status "Deleting Lumosana deployment from namespace: $NAMESPACE"
    
    if [ "$DRY_RUN" = false ]; then
        kubectl delete namespace "$NAMESPACE" --ignore-not-found=true
        print_success "Deployment deleted successfully"
    else
        print_status "DRY RUN: Would delete namespace '$NAMESPACE'"
    fi
}

# Wait for deployments to be ready
wait_for_deployments() {
    if [ "$DRY_RUN" = true ]; then
        print_status "DRY RUN: Would wait for deployments to be ready"
        return
    fi
    
    print_status "Waiting for deployments to be ready..."
    
    local deployments=(
        "transaction-debugger"
        "envoy-proxy"
    )
    
    for deployment in "${deployments[@]}"; do
        print_status "Waiting for deployment/$deployment to be ready..."
        kubectl wait --for=condition=available --timeout=300s deployment/"$deployment" -n "$NAMESPACE" || {
            print_warning "Deployment $deployment may not be ready yet"
        }
    done
    
    print_success "All deployments are ready"
}

# Show deployment status
show_status() {
    if [ "$DRY_RUN" = true ]; then
        return
    fi
    
    print_status "Deployment Status:"
    echo ""
    
    echo "Pods:"
    kubectl get pods -n "$NAMESPACE"
    echo ""
    
    echo "Services:"
    kubectl get services -n "$NAMESPACE"
    echo ""
    
    echo "Deployments:"
    kubectl get deployments -n "$NAMESPACE"
    echo ""
}

# Show access instructions
show_access_info() {
    if [ "$DRY_RUN" = true ]; then
        return
    fi
    
    print_success "Lumosana deployed successfully!"
    echo ""
    echo "🌐 Access Instructions:"
    echo ""
    echo "📋 Transaction Debugger:"
    echo "   kubectl port-forward svc/transaction-debugger-lb 8080:80 -n $NAMESPACE"
    echo "   Then open: http://localhost:8080"
    echo ""
    echo "🔧 Envoy Proxy:"
    echo "   kubectl port-forward svc/envoy-proxy 10000:80 -n $NAMESPACE"
    echo "   Then open: http://localhost:10000"
    echo ""
    echo "📝 To view logs:"
    echo "   kubectl logs -f deployment/transaction-debugger -n $NAMESPACE"
    echo ""
    echo "🗑️  To delete the deployment:"
    echo "   $0 --delete -n $NAMESPACE"
    echo ""
}

# Main deployment flow
main() {
    echo ""
    echo "🚀 Lumosana Kubernetes Deployment"
    echo "=================================="
    echo ""
    
    if [ "${DELETE_MODE:-false}" = true ]; then
        delete_deployment
        exit 0
    fi
    
    check_prerequisites
    set_context
    build_image
    setup_namespace
    apply_manifests
    wait_for_deployments
    show_status
    show_access_info
    
    print_success "Deployment completed! 🎉"
}

# Run main function
main "$@"
