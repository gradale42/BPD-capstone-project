use actix_web::HttpResponse;
use serde_json::json;
use crate::bitcoin::node_manager::BitcoinNodeManager;

pub mod rpc;
pub mod node_manager;