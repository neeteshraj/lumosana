mod routes;
mod grpc;
mod models;
mod controllers;
mod services;
mod dtos;
mod utils;
mod config;

use actix_web::{App, HttpServer, middleware::Logger, web};
use actix_cors::Cors;
use tonic::transport::Server;
use grpc::transaction::debugger::transaction_debugger_server::TransactionDebuggerServer;
use grpc::health::debugger::health_check_server::HealthCheckServer;
use grpc::{TransactionDebuggerService, HealthCheckService};
use config::Config;
use std::env;
use utils::telemetry::{init_telemetry, shutdown_telemetry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let args: Vec<String> = env::args().collect();

    let config = Config::from_env()
        .map_err(|e| format!("Failed to load configuration: {}", e))?;

    config.validate()
        .map_err(|e| format!("Configuration validation failed: {}", e))?;

    if args.len() > 1 {
        match args[1].as_str() {
            "--print-detailed" => {
                config.print_detailed();
                return Ok(());
            }
            "--save-config" => {
                if args.len() < 3 {
                    eprintln!("Error: --save-config requires a file path");
                    eprintln!("Usage: {} --save-config <file_path>", args[0]);
                    std::process::exit(1);
                }
                config.save_to_file(&args[2])?;
                return Ok(());
            }
            "--help" => {
                println!("Transaction Debugger");
                println!("Usage: {} [OPTIONS]", args[0]);
                println!("\nOptions:");
                println!("  --print-detailed    Print detailed configuration in JSON format");
                println!("  --save-config FILE  Save configuration to specified file");
                println!("  --help              Show this help message");
                println!("\nWithout options, starts the full server.");
                return Ok(());
            }
            "--version" => {
                println!("{} v{}", config.app.name, config.app.version);
                return Ok(());
            }
            _ => {
                eprintln!("Error: Unknown command-line argument '{}'", args[1]);
                eprintln!("Use --help for usage information.");
                std::process::exit(1);
            }
        }
    }

    config.print_summary();

    // === INIT TELEMETRY ===
    let (tracer_provider, meter_provider, logger_provider) = init_telemetry();
    println!("Starting Transaction Debugger Service");

    let grpc_addr = format!("{}:{}", config.server.host, config.server.grpc_port).parse()?;
    let transaction_service = TransactionDebuggerService;
    let health_service = HealthCheckService;

    println!("Starting gRPC server on {}", grpc_addr);
    let grpc_server = Server::builder()
        .add_service(TransactionDebuggerServer::new(transaction_service))
        .add_service(HealthCheckServer::new(health_service))
        .serve(grpc_addr);

    let http_addr = format!("{}:{}", config.server.host, config.server.http_port);
    println!("Starting HTTP server on {}", http_addr);

    let cors_origins = config.security.cors_allowed_origins.clone();
    let cors_methods = config.security.cors_allowed_methods.clone();
    let cors_headers = config.security.cors_allowed_headers.clone();

    let http_server = HttpServer::new(move || {
        let mut cors = Cors::default();

        let origins: Vec<&str> = cors_origins.split(',').collect();
        let methods: Vec<&str> = cors_methods.split(',').collect();
        let headers: Vec<&str> = cors_headers.split(',').collect();

        if origins.contains(&"*") {
            cors = cors.allow_any_origin();
        } else {
            for origin in &origins {
                cors = cors.allowed_origin(origin.trim());
            }
        }

        if methods.contains(&"*") {
            cors = cors.allow_any_method();
        }

        if headers.contains(&"*") {
            cors = cors.allow_any_header();
        }

        App::new()
            .app_data(web::Data::new(std::sync::Arc::new(config.clone())))
            .wrap(Logger::default())
            .wrap(cors)
            .configure(routes::api::config)
    })
        .bind(&http_addr)?
        .run();

    let result = tokio::try_join!(
        async { grpc_server.await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>) },
        async { http_server.await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>) }
    );

    if let Err(err) = shutdown_telemetry(tracer_provider, meter_provider, logger_provider) {
        eprintln!("Telemetry shutdown failed: {err}");
    }

    result?;

    Ok(())
}
