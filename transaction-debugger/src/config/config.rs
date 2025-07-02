use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub observability: ObservabilityConfig,
    pub jaeger: JaegerConfig,
    pub otel: OtelConfig,
    pub prometheus: PrometheusConfig,
    pub health: HealthConfig,
    pub performance: PerformanceConfig,
    pub database: DatabaseConfig,
    pub solana: SolanaConfig,
    pub security: SecurityConfig,
    pub development: DevelopmentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub http_port: u16,
    pub grpc_port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub rust_log: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub enable_metrics: bool,
    pub enable_tracing: bool,
    pub enable_logging: bool,
    pub trace_sampling_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JaegerConfig {
    pub agent_host: String,
    pub agent_port: u16,
    pub collector_endpoint: String,
    pub exporter_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelConfig {
    pub service_name: String,
    pub service_version: String,
    pub resource_attributes: String,
    pub exporter_endpoint: String,
    pub traces_endpoint: String,
    pub metrics_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusConfig {
    pub endpoint: String,
    pub metrics_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    pub check_interval: u64,
    pub check_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub request_timeout: u64,
    pub max_concurrent_requests: usize,
    pub worker_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: Option<String>,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub rpc_timeout: u64,
    pub websocket_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub cors_allowed_origins: String,
    pub cors_allowed_methods: String,
    pub cors_allowed_headers: String,
    pub api_key_header: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentConfig {
    pub debug_mode: bool,
    pub hot_reload: bool,
    pub pretty_logs: bool,
}

#[derive(Debug)]
pub enum ConfigError {
    ParseError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}
impl std::error::Error for ConfigError {}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        // Try to load environment-specific .env file first, then fallback to .env
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let env_file = format!(".env.{}", environment);
        
        // Try to load environment-specific file first
        if let Ok(_) = dotenvy::from_filename(&env_file) {
            println!("Loaded environment configuration from: {}", env_file);
        } else if let Ok(_) = dotenvy::dotenv() {
            println!("Loaded environment configuration from: .env");
        } else {
            println!("Warning: No .env file found (tried {} and .env), using environment variables only", env_file);
        }

        Ok(Config {
            app: AppConfig {
                name: get_env_var("APP_NAME", "transaction-debugger")?,
                version: get_env_var("APP_VERSION", "1.0.0")?,
                environment: get_env_var("ENVIRONMENT", "development")?,
            },
            server: ServerConfig {
                http_port: get_env_var_parsed("HTTP_PORT", 8080)?,
                grpc_port: get_env_var_parsed("GRPC_PORT", 50051)?,
                host: get_env_var("HOST", "0.0.0.0")?,
            },
            logging: LoggingConfig {
                rust_log: get_env_var("RUST_LOG", "info")?,
            },
            observability: ObservabilityConfig {
                enable_metrics: get_env_var_parsed("ENABLE_METRICS", true)?,
                enable_tracing: get_env_var_parsed("ENABLE_TRACING", true)?,
                enable_logging: get_env_var_parsed("ENABLE_LOGGING", true)?,
                trace_sampling_rate: get_env_var_parsed("TRACE_SAMPLING_RATE", 1.0)?,
            },
            jaeger: JaegerConfig {
                agent_host: get_env_var("JAEGER_AGENT_HOST", "localhost")?,
                agent_port: get_env_var_parsed("JAEGER_AGENT_PORT", 6831)?,
                collector_endpoint: get_env_var("JAEGER_COLLECTOR_ENDPOINT", "http://localhost:14268/api/traces")?,
                exporter_endpoint: get_env_var("OTEL_EXPORTER_JAEGER_ENDPOINT", "http://localhost:14268/api/traces")?,
            },
            otel: OtelConfig {
                service_name: get_env_var("OTEL_SERVICE_NAME", "transaction-debugger")?,
                service_version: get_env_var("OTEL_SERVICE_VERSION", "1.0.0")?,
                resource_attributes: get_env_var("OTEL_RESOURCE_ATTRIBUTES", "service.name=transaction-debugger,service.version=1.0.0")?,
                exporter_endpoint: get_env_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4317")?,
                traces_endpoint: get_env_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://localhost:4317")?,
                metrics_endpoint: get_env_var("OTEL_EXPORTER_OTLP_METRICS_ENDPOINT", "http://localhost:4317")?,
            },
            prometheus: PrometheusConfig {
                endpoint: get_env_var("PROMETHEUS_ENDPOINT", "localhost:9090")?,
                metrics_port: get_env_var_parsed("METRICS_PORT", 9464)?,
            },
            health: HealthConfig {
                check_interval: get_env_var_parsed("HEALTH_CHECK_INTERVAL", 30)?,
                check_timeout: get_env_var_parsed("HEALTH_CHECK_TIMEOUT", 5)?,
            },
            performance: PerformanceConfig {
                request_timeout: get_env_var_parsed("REQUEST_TIMEOUT", 30)?,
                max_concurrent_requests: get_env_var_parsed("MAX_CONCURRENT_REQUESTS", 1000)?,
                worker_threads: get_env_var_parsed("WORKER_THREADS", 4)?,
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").ok(),
                max_connections: get_env_var_parsed("DATABASE_MAX_CONNECTIONS", 10)?,
                min_connections: get_env_var_parsed("DATABASE_MIN_CONNECTIONS", 5)?,
            },
            solana: SolanaConfig {
                rpc_url: get_env_var("SOLANA_RPC_URL", "https://api.mainnet-beta.solana.com")?,
                rpc_timeout: get_env_var_parsed("SOLANA_RPC_TIMEOUT", 30)?,
                websocket_url: get_env_var("SOLANA_WEBSOCKET_URL", "wss://api.mainnet-beta.solana.com")?,
            },
            security: SecurityConfig {
                cors_allowed_origins: get_env_var("CORS_ALLOWED_ORIGINS", "*")?,
                cors_allowed_methods: get_env_var("CORS_ALLOWED_METHODS", "GET,POST,PUT,DELETE,OPTIONS")?,
                cors_allowed_headers: get_env_var("CORS_ALLOWED_HEADERS", "*")?,
                api_key_header: env::var("API_KEY_HEADER").ok(),
            },
            development: DevelopmentConfig {
                debug_mode: get_env_var_parsed("DEBUG_MODE", false)?,
                hot_reload: get_env_var_parsed("HOT_RELOAD", false)?,
                pretty_logs: get_env_var_parsed("PRETTY_LOGS", true)?,
            },
        })
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server.http_port == 0 {
            return Err(ConfigError::ParseError("HTTP port cannot be 0".to_string()));
        }
        
        if self.server.grpc_port == 0 {
            return Err(ConfigError::ParseError("gRPC port cannot be 0".to_string()));
        }

        if self.observability.trace_sampling_rate < 0.0 || self.observability.trace_sampling_rate > 1.0 {
            return Err(ConfigError::ParseError("Trace sampling rate must be between 0.0 and 1.0".to_string()));
        }

        Ok(())
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn print_summary(&self) {
        println!("=== Configuration Summary ===");
        println!("App: {} v{} ({})", self.app.name, self.app.version, self.app.environment);
        println!("HTTP Server: {}:{}", self.server.host, self.server.http_port);
        println!("gRPC Server: {}:{}", self.server.host, self.server.grpc_port);
        println!("Log Level: {}", self.logging.rust_log);
        println!("Observability: metrics={}, tracing={}, logging={}", 
                 self.observability.enable_metrics, 
                 self.observability.enable_tracing, 
                 self.observability.enable_logging);
        println!("============================");
    }

    pub fn print_detailed(&self) {
        println!("=== Detailed Configuration ===");
        if let Ok(json) = self.to_json() {
            println!("{}", json);
        } else {
            println!("Failed to serialize configuration to JSON");
        }
        println!("===============================");
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        println!("Configuration saved to: {}", path);
        Ok(())
    }
}

fn get_env_var(key: &str, default: &str) -> Result<String, ConfigError> {
    match env::var(key) {
        Ok(val) => Ok(val),
        Err(_) => {
            Ok(default.to_string())
        }
    }
}

fn get_env_var_parsed<T>(key: &str, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr + std::fmt::Display + Copy,
    T::Err: std::fmt::Display,
{
    match env::var(key) {
        Ok(val) => val.parse().map_err(|e| {
            ConfigError::ParseError(format!("Failed to parse {}: {} ({})", key, val, e))
        }),
        Err(_) => Ok(default),
    }
}
