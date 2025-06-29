# Solana Transaction Debugger

A comprehensive Rust-based service for analyzing and debugging Solana blockchain transactions. This service provides both HTTP REST API and gRPC interfaces with distributed tracing capabilities.

## Features

- **Transaction Analysis**: Deep analysis of Solana transactions including:
  - Success/failure status
  - Compute units consumed
  - Fee analysis
  - Account involvement tracking
  - Program interaction analysis
  - Balance changes
  - Log message extraction

- **Multiple Interfaces**:
  - REST HTTP API
  - gRPC API
  - Health check endpoint

- **Observability**:
  - Distributed tracing with Jaeger
  - Structured logging
  - OpenTelemetry integration

- **Cloud Native**:
  - Kubernetes deployment ready
  - Docker containerized
  - Envoy proxy integration
  - ConfigMap configuration

## API Endpoints

### HTTP REST API

#### Debug Transaction
```bash
POST /debug
Content-Type: application/json

{
  "signature": "transaction_signature_here",
  "rpc_url": "https://api.mainnet-beta.solana.com"
}
```

#### Health Check
```bash
GET /health
```

### gRPC API

The service implements the `TransactionDebugger` service defined in `proto/debugger.proto`:

```protobuf
service TransactionDebugger {
  rpc DebugTransaction (DebugRequest) returns (DebugResponse);
}
```

## Quick Start

### Prerequisites

- Rust 1.77+
- Docker (optional)
- Kubernetes cluster (optional)

### Local Development

1. **Clone and build**:
```bash
cargo build --release
```

2. **Run the service**:
```bash
cargo run
```

The service will start with:
- HTTP API on port 8080
- gRPC API on port 50051

3. **Test the API**:
```bash
curl -X POST http://localhost:8080/debug \
  -H "Content-Type: application/json" \
  -d '{
    "signature": "your_transaction_signature",
    "rpc_url": "https://api.mainnet-beta.solana.com"
  }'
```

### Docker Deployment

1. **Build the Docker image**:
```bash
docker build -t transaction-debugger .
```

2. **Run the container**:
```bash
docker run -p 8080:8080 -p 50051:50051 transaction-debugger
```

### Kubernetes Deployment

1. **Deploy Jaeger (for tracing)**:
```bash
kubectl apply -f k8s/jaeger.yaml
```

2. **Deploy the application**:
```bash
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
```

3. **Deploy Envoy proxy (optional)**:
```bash
kubectl apply -f k8s/envoy.yaml
```

## Configuration

The service can be configured using environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `HTTP_PORT` | `8080` | HTTP server port |
| `GRPC_PORT` | `50051` | gRPC server port |
| `RUST_LOG` | `info` | Log level |
| `JAEGER_SERVICE_NAME` | `transaction-debugger` | Service name for tracing |
| `JAEGER_AGENT_HOST` | `localhost` | Jaeger agent hostname |
| `JAEGER_AGENT_PORT` | `6831` | Jaeger agent port |

## Response Format

The debug endpoint returns detailed transaction analysis:

```json
{
  "signature": "transaction_signature",
  "slot": 123456789,
  "block_time": 1640995200,
  "transaction": { /* Full transaction data */ },
  "meta": { /* Transaction status */ },
  "analysis": {
    "success": true,
    "error": null,
    "compute_units_consumed": 200000,
    "fee": 5000,
    "accounts_involved": ["pubkey1", "pubkey2"],
    "program_ids": ["program1", "program2"],
    "instruction_count": 3,
    "pre_balances": [1000000, 500000],
    "post_balances": [995000, 500000],
    "log_messages": ["Program log: Success"]
  }
}
```

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   HTTP Client   │    │   gRPC Client   │    │  Envoy Proxy    │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          ▼                      ▼                      ▼
┌─────────────────────────────────────────────────────────────────┐
│              Transaction Debugger Service                       │
│  ┌─────────────────┐              ┌─────────────────┐           │
│  │   HTTP Server   │              │   gRPC Server   │           │
│  │   (Actix-Web)   │              │     (Tonic)     │           │
│  └─────────┬───────┘              └─────────┬───────┘           │
│            │                                │                   │
│            ▼                                ▼                   │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                Transaction Analyzer                         ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
            ┌─────────────────────────────────┐
            │         Solana RPC              │
            │   (api.mainnet-beta.solana.com) │
            └─────────────────────────────────┘
```

## Monitoring and Observability

The service includes comprehensive observability features:

- **Distributed Tracing**: Integration with Jaeger for request tracing
- **Structured Logging**: JSON formatted logs with correlation IDs
- **Health Checks**: Kubernetes-ready liveness and readiness probes
- **Metrics**: Built-in performance metrics

Access Jaeger UI at `http://jaeger-ui-service/` when deployed in Kubernetes.

## Development

### Project Structure

```
transaction-debugger/
├── src/
│   ├── main.rs           # Application entry point
│   ├── api.rs            # HTTP REST API handlers
│   ├── grpc.rs           # gRPC service implementation
│   ├── debugger.rs       # Core transaction analysis logic
│   ├── models.rs         # Data models and structures
│   └── tracing.rs        # Observability setup
├── proto/
│   └── debugger.proto    # gRPC service definition
├── k8s/                  # Kubernetes manifests
├── Dockerfile            # Container image definition
├── envoy.yaml           # Envoy proxy configuration
└── Cargo.toml           # Rust dependencies
```

### Adding New Features

1. **New Analysis Features**: Add logic to `debugger.rs` and update `TransactionAnalysis` in `models.rs`
2. **New API Endpoints**: Add handlers to `api.rs`
3. **New gRPC Methods**: Update `proto/debugger.proto` and implement in `grpc.rs`

### Testing

```bash
# Run unit tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Test with real transaction
curl -X POST http://localhost:8080/debug \
  -H "Content-Type: application/json" \
  -d '{
    "signature": "real_transaction_signature_from_solana",
    "rpc_url": "https://api.mainnet-beta.solana.com"
  }'
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License.
