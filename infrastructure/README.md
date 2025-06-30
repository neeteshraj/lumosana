# Lumosana Infrastructure

This directory contains all infrastructure-related configurations and scripts for the Lumosana project.

## 📁 Directory Structure

```
infrastructure/
├── docker/              # Docker Compose configurations
│   └── docker-compose.yml
├── envoy/              # Envoy proxy configuration
│   └── envoy.yaml
├── k8s/                # Kubernetes manifests
│   ├── configmap.yaml
│   ├── deployment.yaml
│   ├── envoy.yaml
│   ├── jaeger.yaml
│   └── service.yaml
├── observability/      # Monitoring and observability configs
│   ├── grafana/
│   │   ├── dashboards/
│   │   ├── dashboards.yml
│   │   └── datasources.yml
│   ├── otel-collector.yml
│   └── prometheus.yml
└── scripts/           # Infrastructure automation scripts
    ├── start-observability.sh
    └── test-envoy-features.sh
```

## 🚀 Quick Start

### Docker Compose (Recommended for Development)

1. **Start the full stack:**
   ```bash
   cd infrastructure/scripts
   ./start-observability.sh
   ```

2. **Or manually:**
   ```bash
   cd infrastructure/docker
   docker-compose up -d
   ```

### Kubernetes Deployment

1. **Deploy to Kubernetes:**
   ```bash
   kubectl apply -f infrastructure/k8s/
   ```

## 🌐 Service URLs

When running with Docker Compose:

- **Lumosana Debugger API**: http://localhost:8080
- **API Documentation**: http://localhost:8080/swagger-ui/
- **Envoy Proxy**: http://localhost:10000
- **Envoy Admin**: http://localhost:9901
- **Grafana Dashboard**: http://localhost:3000 (admin/admin)
- **Prometheus**: http://localhost:9090
- **Jaeger Tracing**: http://localhost:16686

## 🔧 Configuration

### Environment Variables

The service can be configured using environment variables defined in:
- `docker/docker-compose.yml` for Docker Compose
- `k8s/configmap.yaml` for Kubernetes

### Observability Stack

The infrastructure includes a complete observability stack:

- **Prometheus**: Metrics collection and storage
- **Grafana**: Metrics visualization and dashboards
- **Jaeger**: Distributed tracing
- **OpenTelemetry Collector**: Telemetry data collection and processing
- **Envoy Proxy**: Advanced load balancing and traffic management

### Envoy Features

The Envoy proxy is configured with advanced features:
- Circuit breakers
- Health checks
- Rate limiting
- Outlier detection
- Access logging
- Fault injection
- CORS support
- gRPC Web support
- Comprehensive metrics

## 📊 Monitoring

### Grafana Dashboards

Pre-configured dashboards are available in `observability/grafana/dashboards/`:
- Transaction Debugger metrics
- System performance
- Request/response patterns

### Prometheus Metrics

The following services expose Prometheus metrics:
- Lumosana Debugger (`/metrics`)
- Envoy Proxy (`/stats/prometheus`)
- OpenTelemetry Collector

## 🛠️ Scripts

### `start-observability.sh`
Automated script to start the complete observability stack with health checks.

### `test-envoy-features.sh`
Comprehensive test script to verify all Envoy features are working correctly.

## 🔄 Development Workflow

1. **Start infrastructure:**
   ```bash
   ./infrastructure/scripts/start-observability.sh
   ```

2. **Develop and test:**
   - Make changes to the application code
   - Access services through Envoy proxy at `localhost:10000`
   - Monitor metrics in Grafana
   - View traces in Jaeger

3. **Test Envoy features:**
   ```bash
   ./infrastructure/scripts/test-envoy-features.sh
   ```

4. **Stop infrastructure:**
   ```bash
   cd infrastructure/docker
   docker-compose down
   ```

## 📝 Notes

- The Docker Compose configuration automatically builds the Lumosana Debugger from the parent directory
- All volumes are configured for data persistence
- Services are configured with proper health checks and dependencies
- The infrastructure supports both HTTP REST and gRPC APIs
- Comprehensive logging and metrics are enabled by default

## 🔧 Customization

To customize the infrastructure:

1. **Modify service configuration**: Edit files in respective directories
2. **Add new services**: Update `docker/docker-compose.yml`
3. **Configure monitoring**: Modify files in `observability/`
4. **Adjust Envoy settings**: Edit `envoy/envoy.yaml`

For production deployments, consider:
- Using external databases for Prometheus and Grafana
- Implementing proper secret management
- Configuring resource limits and requests
- Setting up proper backup strategies
