#!/bin/bash

# Lumosana - Observability Stack Startup Script

set -e

echo "🚀 Starting Lumosana with full observability stack..."

# Check if Docker and Docker Compose are available
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Change to the docker directory
cd "$(dirname "$0")/../docker"

# Create necessary directories
echo "📁 Creating necessary directories..."
mkdir -p ../observability/grafana/dashboards

# Start the observability stack
echo "🔧 Starting observability stack..."
docker-compose up -d

# Wait for services to start...
echo "⏳ Waiting for services to start..."
sleep 10

# Check service health
echo "🏥 Checking service health..."

# Wait for Grafana to be ready
echo "📊 Waiting for Grafana to be ready..."
timeout 60 bash -c 'until curl -f -s http://localhost:3000/api/health; do sleep 2; done' || {
    echo "❌ Grafana failed to start"
    exit 1
}

# Wait for Prometheus to be ready
echo "📈 Waiting for Prometheus to be ready..."
timeout 60 bash -c 'until curl -f -s http://localhost:9090/-/ready; do sleep 2; done' || {
    echo "❌ Prometheus failed to start"
    exit 1
}

# Wait for Jaeger to be ready
echo "🔍 Waiting for Jaeger to be ready..."
timeout 60 bash -c 'until curl -f -s http://localhost:16686/; do sleep 2; done' || {
    echo "❌ Jaeger failed to start"
    exit 1
}

echo ""
echo "✅ All services are running!"
echo ""
echo "🌐 Service URLs:"
echo "  📊 Grafana Dashboard:      http://localhost:3000 (admin/admin)"
echo "  📈 Prometheus:             http://localhost:9090"
echo "  🔍 Jaeger Tracing:         http://localhost:16686"
echo "  📋 Lumosana Debugger:      http://localhost:8080"
echo "  📖 API Documentation:      http://localhost:8080/swagger-ui/"
echo "  📊 Metrics Endpoint:       http://localhost:8080/metrics"
echo "  🔧 Envoy Proxy:            http://localhost:10000"
echo "  🔧 Envoy Admin:            http://localhost:9901"
echo ""
echo "🔧 To stop all services:"
echo "  docker-compose down"
echo ""
echo "📝 To view logs:"
echo "  docker-compose logs -f [service-name]"
echo ""
echo "🎯 Ready for transaction debugging with full observability!"
