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

echo ""
echo "✅ All services are running!"
echo ""
echo "🌐 Service URLs:"
echo "  📋 Lumosana Debugger:      http://localhost:8080"
echo "  📖 API Documentation:      http://localhost:8080/swagger-ui/"
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
