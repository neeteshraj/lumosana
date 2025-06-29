use actix_web::{web, post, HttpResponse, Responder, Result};
use crate::debugger::analyze_transaction;
use crate::models::DebugRequest;
use tracing::instrument;

#[post("/debug")]
#[instrument]
pub async fn debug_tx(req: web::Json<DebugRequest>) -> Result<impl Responder> {
    match analyze_transaction(&req.signature, &req.rpc_url).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e,
            "message": "Failed to analyze transaction"
        }))),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(debug_tx);
}
