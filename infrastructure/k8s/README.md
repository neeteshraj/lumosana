# Lumosana Kubernetes Deployment

This directory contains Kubernetes manifests and deployment scripts for the complete Lumosana observability stack.

## 📁 Structure

```
infrastructure/k8s/
├── configmap.yaml                  # Transaction debugger configuration
├── deployment.yaml                 # Transaction debugger deployment
├── service.yaml                   # Transaction debugger services
├── envoy.yaml                     # Envoy proxy configuration and deployment
├── jaeger.yaml                    # Jaeger tracing deployment and services
├── prometheus-config.yaml         # Prometheus configuration
├── prometheus-deployment.yaml     # Prometheus deployment
├── prometheus-service.yaml        # Prometheus services
├── otel-collector-config.yaml     # OpenTelemetry Collector configuration
├── otel-collector-deployment.yaml # OpenTelemetry Collector deployment
├── otel-collector-service.yaml    # OpenTelemetry Collector service
├── grafana-config.yaml            # Grafana configuration and datasources
├── grafana-deployment.yaml        # Grafana deployment
├── grafana-service.yaml           # Grafana services
└── README.md                      # This file

infrastructure/scripts/
└── deploy-k8s.sh                  # Comprehensive deployment script
```

## 🚀 Quick Start

### Prerequisites

- Kubernetes cluster (local or remote)
- `kubectl` configured and connected to your cluster
- Docker (for building images)

### Deploy Everything

```bash
# Deploy to default 'lumosana' namespace
./infrastructure/scripts/deploy-k8s.sh

# Deploy to custom namespace
./infrastructure/scripts/deploy-k8s.sh -n production

# Dry run to see what would be deployed
./infrastructure/scripts/deploy-k8s.sh --dry-run

# Skip Docker image build (if image already exists)
./infrastructure/scripts/deploy-k8s.sh --skip-build
```

### Delete Deployment

```bash
# Delete the entire deployment
./infrastructure/scripts/deploy-k8s.sh --delete

# Delete from custom namespace
./infrastructure/scripts/deploy-k8s.sh --delete -n production
```

## 🔧 Manual Deployment

If you prefer to deploy components individually:

```bash
# Create namespace
kubectl create namespace lumosana

# Deploy in order (respecting dependencies)
kubectl apply -f k8s/configmap.yaml -n lumosana
kubectl apply -f k8s/jaeger.yaml -n lumosana
kubectl apply -f k8s/prometheus-config.yaml -n lumosana
kubectl apply -f k8s/prometheus-deployment.yaml -n lumosana
kubectl apply -f k8s/prometheus-service.yaml -n lumosana
kubectl apply -f k8s/otel-collector-config.yaml -n lumosana
kubectl apply -f k8s/otel-collector-deployment.yaml -n lumosana
kubectl apply -f k8s/otel-collector-service.yaml -n lumosana
kubectl apply -f k8s/grafana-config.yaml -n lumosana
kubectl apply -f k8s/grafana-deployment.yaml -n lumosana
kubectl apply -f k8s/grafana-service.yaml -n lumosana
kubectl apply -f k8s/deployment.yaml -n lumosana
kubectl apply -f k8s/service.yaml -n lumosana
kubectl apply -f k8s/envoy.yaml -n lumosana
```

## 🌐 Accessing Services

After deployment, you can access services using port-forwarding:

### Grafana Dashboard
```bash
kubectl port-forward svc/grafana-lb 3000:80 -n lumosana
# Open: http://localhost:3000 (admin/admin)
```

### Prometheus
```bash
kubectl port-forward svc/prometheus-lb 9090:80 -n lumosana
# Open: http://localhost:9090
```

### Jaeger Tracing
```bash
kubectl port-forward svc/jaeger-ui 16686:80 -n lumosana
# Open: http://localhost:16686
```

### Transaction Debugger
```bash
kubectl port-forward svc/transaction-debugger-lb 8080:80 -n lumosana
# Open: http://localhost:8080
```

### Envoy Proxy
```bash
kubectl port-forward svc/envoy-proxy 10000:80 -n lumosana
# Open: http://localhost:10000
```

## 📊 Components

### Core Service
- **Transaction Debugger**: Main Rust service with HTTP and gRPC endpoints

### Observability Stack
- **Jaeger**: Distributed tracing
- **Prometheus**: Metrics collection
- **Grafana**: Visualization and dashboards
- **OpenTelemetry Collector**: Telemetry data processing

### Infrastructure
- **Envoy Proxy**: Load balancing and observability
- **ConfigMaps**: Configuration management

## 🔍 Monitoring & Debugging

### Check Deployment Status
```bash
kubectl get all -n lumosana
```

### View Logs
```bash
# Transaction debugger logs
kubectl logs -f deployment/transaction-debugger -n lumosana

# Jaeger logs
kubectl logs -f deployment/jaeger -n lumosana

# Prometheus logs
kubectl logs -f deployment/prometheus -n lumosana

# Grafana logs
kubectl logs -f deployment/grafana -n lumosana
```

### Debug Issues
```bash
# Describe pods to see events and status
kubectl describe pods -n lumosana

# Check service endpoints
kubectl get endpoints -n lumosana

# Check ingress/service connectivity
kubectl exec -it deployment/transaction-debugger -n lumosana -- curl http://prometheus-service:9090/-/ready
```

## ⚙️ Configuration

### Environment Variables
Configure the transaction debugger through the ConfigMap in `configmap.yaml`:

- `HTTP_PORT`: HTTP server port (default: 8080)
- `GRPC_PORT`: gRPC server port (default: 50051)
- `RUST_LOG`: Logging level (default: info)
- `JAEGER_SERVICE_NAME`: Service name for tracing
- `JAEGER_AGENT_HOST`: Jaeger agent hostname
- `OTEL_*`: OpenTelemetry configuration

### Resource Limits
Each deployment includes resource requests and limits. Adjust them in the respective YAML files based on your cluster capacity:

```yaml
resources:
  requests:
    memory: "128Mi"
    cpu: "100m"
  limits:
    memory: "512Mi"
    cpu: "500m"
```

## 🔄 Updates

To update the deployment:

1. Build new Docker image:
   ```bash
   cd transaction-debugger
   docker build -t lumosana/transaction-debugger:latest .
   ```

2. Update the deployment:
   ```bash
   kubectl rollout restart deployment/transaction-debugger -n lumosana
   ```

## 🧹 Cleanup

Remove everything:
```bash
kubectl delete namespace lumosana
```

Or use the deployment script:
```bash
./infrastructure/scripts/deploy-k8s.sh --delete
```

## 📝 Notes

- The deployment script automatically handles dependencies and deployment order
- Services use ClusterIP for internal communication and LoadBalancer for external access
- All configuration is managed through Kubernetes ConfigMaps
- The deployment includes health checks and resource limits
- Persistent volumes are not configured by default (suitable for development/testing)
