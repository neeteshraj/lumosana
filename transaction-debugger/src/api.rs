use actix_web::{web};
use crate::controllers::{HealthController, TransactionController};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/debug", web::post().to(TransactionController::debug_transaction))
        .route("/health-detailed", web::post().to(HealthController::detailed_health_check));
}
