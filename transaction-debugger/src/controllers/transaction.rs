use actix_web::{web, HttpResponse, Responder, Result};
use crate::services::TransactionService;
use crate::dtos::DebugRequestDto;
use tracing::instrument;

pub struct TransactionController;

impl TransactionController {
    #[instrument]
    pub async fn debug_transaction(req: web::Json<DebugRequestDto>) -> Result<impl Responder> {
        match TransactionService::analyze_transaction(req.into_inner()).await {
            Ok(response) => {
                Ok(HttpResponse::Ok().json(response))
            },
            Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e,
                "message": "Failed to analyze transaction"
            }))),
        }
    }
}
