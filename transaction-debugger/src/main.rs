mod api;
mod debugger;
mod grpc;
mod models;
mod tracing;
mod controllers;
mod services;
mod dtos;
mod utils;

use actix_web::{web, App, HttpServer, middleware::Logger};
use tonic::transport::Server;
use grpc::transaction::debugger::transaction_debugger_server::TransactionDebuggerServer;
use grpc::health::debugger::health_check_server::HealthCheckServer;
use grpc::{TransactionDebuggerService, HealthCheckService};
use controllers::HealthController;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::init_tracing();
    
    println!("Starting Transaction Debugger Service");

    let grpc_port = env::var("GRPC_PORT").unwrap_or_else(|_| "50051".to_string());
    let http_port = env::var("HTTP_PORT").unwrap_or_else(|_| "8080".to_string());

    let grpc_addr = format!("0.0.0.0:{}", grpc_port).parse()?;
    let transaction_service = TransactionDebuggerService;
    let health_service = HealthCheckService;
    
    println!("Starting gRPC server on {}", grpc_addr);
    let grpc_server = Server::builder()
        .add_service(TransactionDebuggerServer::new(transaction_service))
        .add_service(HealthCheckServer::new(health_service))
        .serve(grpc_addr);

    let http_addr = format!("0.0.0.0:{}", http_port);
    println!("Starting HTTP server on {}", http_addr);
    
    let http_server = HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .configure(api::config)
            .route("/health", web::get().to(HealthController::health_check))
    })
    .bind(&http_addr)?
    .run();

    tokio::try_join!(
        async { grpc_server.await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>) },
        async { http_server.await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>) }
    )?;

    Ok(())
}
