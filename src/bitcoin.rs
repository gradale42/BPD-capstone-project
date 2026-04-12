use actix_web::HttpResponse;
use serde_json::json;
use crate::bitcoin::node_manager::BitcoinNodeManager;

pub mod rpc;
pub mod node_manager;

pub async fn execute_rpc<F, R>(node_manager: &BitcoinNodeManager, wallet_name: &str, f: F) -> HttpResponse
where
    F: FnOnce(&bitcoincore_rpc::Client) -> Result<R, bitcoincore_rpc::Error> + Send + 'static,
    R: serde::Serialize + Send + 'static,
{
    match node_manager.execute_rpc(wallet_name, f).await {
        Ok(data) => HttpResponse::Ok().json(json!({
                "status": "success",
                "data": data
            })),
        Err(err) => HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": err
            })),
    }
}