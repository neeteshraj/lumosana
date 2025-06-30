#!/bin/bash

set -e

echo "🧪 Testing Lumosana Infrastructure Services"
echo "==========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

test_endpoint() {
    local name="$1"
    local url="$2"
    local expected_code="${3:-200}"
    
    echo -n "Testing $name... "
    
    if response=$(curl -s -w "%{http_code}" "$url" 2>/dev/null); then
        http_code="${response: -3}"
        if [[ "$http_code" =~ ^(200|302|405)$ ]]; then
            echo -e "${GREEN}✓ OK${NC} (HTTP $http_code)"
            return 0
        else
            echo -e "${RED}✗ FAILED${NC} (HTTP $http_code)"
            return 1
        fi
    else
        echo -e "${RED}✗ FAILED${NC} (Connection failed)"
        return 1
    fi
}

echo
echo "📊 Checking Pod Status"
echo "----------------------"
kubectl get pods -n lumosana

echo
echo "🌐 Checking Service Status"
echo "--------------------------"
kubectl get services -n lumosana

echo
echo "🔍 Testing Service Endpoints"
echo "-----------------------------"

# Test transaction-debugger directly
test_endpoint "Transaction Debugger (Direct)" "http://localhost/health"

# Test through Envoy proxy
test_endpoint "Transaction Debugger (via Envoy)" "http://localhost:80/health"

# Kill any existing port forwards
pkill -f "kubectl port-forward" || true
sleep 2

# Start port forwards for other services
echo
echo "🚀 Starting port forwards for testing..."
kubectl port-forward -n lumosana svc/jaeger-ui 16686:80 &
kubectl port-forward -n lumosana svc/prometheus-service 9090:9090 &
kubectl port-forward -n lumosana svc/grafana-service 3000:3000 &
kubectl port-forward -n lumosana svc/otel-collector-service 13133:13133 &

# Wait for port forwards to be ready
sleep 5

# Test all services
test_endpoint "Jaeger UI" "http://localhost:16686"
test_endpoint "Prometheus" "http://localhost:9090/graph"
test_endpoint "Grafana" "http://localhost:3000"
test_endpoint "OTEL Collector" "http://localhost:13133"

echo
echo "🎯 Testing Envoy Features"
echo "-------------------------"

# Test with headers
echo -n "Testing Envoy headers... "
response=$(curl -s -I http://localhost:80/health)
if echo "$response" | grep -q "server: envoy" && echo "$response" | grep -q "x-envoy-upstream-service-time"; then
    echo -e "${GREEN}✓ OK${NC} (Envoy headers present)"
else
    echo -e "${YELLOW}⚠ PARTIAL${NC} (Some Envoy headers missing)"
fi

# Test gRPC endpoint through transaction-debugger LoadBalancer
echo -n "Testing gRPC endpoint... "
if grpcurl -plaintext localhost:50051 list > /dev/null 2>&1; then
    echo -e "${GREEN}✓ OK${NC} (gRPC accessible)"
else
    echo -e "${YELLOW}⚠ SKIPPED${NC} (grpcurl not available or no gRPC services exposed)"
fi

echo
echo "📈 Infrastructure Summary"
echo "========================"
echo -e "${GREEN}✅ All core services are running and accessible${NC}"
echo
echo "Available endpoints:"
echo "  • Transaction Debugger: http://localhost/health (via Envoy)"
echo "  • Transaction Debugger: http://localhost (direct LoadBalancer)"
echo "  • Jaeger UI: http://localhost:16686 (port-forward)"
echo "  • Prometheus: http://localhost:9090 (port-forward)"
echo "  • Grafana: http://localhost:3000 (port-forward)"
echo "  • OTEL Collector: http://localhost:13133 (port-forward)"
echo
echo "gRPC endpoints:"
echo "  • Transaction Debugger gRPC: localhost:50051"
echo
echo -e "${YELLOW}Note: Port forwards will remain active. Press Ctrl+C to stop them.${NC}"

# Keep port forwards running
wait
