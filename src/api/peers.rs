use crate::api::wrap_response;
use crate::domain::peers::PeerInfo;
use crate::AppState;
use actix_web::{web, Responder};
use anyhow::Context;
use bitcoincore_rpc::RpcApi;

#[derive(Debug, serde::Deserialize)]
pub struct PeersParams {
    pub draw: i32,
    pub start: Option<i32>,
    pub length: Option<i32>,
}

#[derive(Debug, serde::Serialize)]
pub struct DataTableResponse<T> {
    pub draw: i32,
    pub records_total: usize,
    pub records_filtered: usize,
    pub data: Vec<T>,
}

#[utoipa::path(
    get,
    path = "/api/v1/peers",
    responses((status = 200, body = Vec<PeerInfo>)),
    tag = "Peers"
)]
pub async fn get_peers(
    state: web::Data<AppState>,
    web::Query(params): web::Query<PeersParams>,
) -> impl Responder {

    let node_manager = &state.node_manager;

    let result: anyhow::Result<_> = async {
        let rpc_result = node_manager
            .execute_rpc("", move |client| {
                let peers_raw =  client.get_peer_info()?;
                let mut peers = Vec::new();
                for peer in peers_raw {
                    peers.push(PeerInfo {
                        peer_id: peer.id,
                        inbound: peer.inbound,
                        subver: peer.subver.clone(),
                        version: peer.version,
                        bytes_sent_total: peer.bytessent,
                        bytes_recv_total: peer.bytesrecv,
                        bytes_sent_delta: 0,
                        bytes_recv_delta: 0,
                        ping: peer.pingtime.unwrap_or(0.0) * 1000.0, // Convert to ms
                    });
                }

                let response = DataTableResponse {
                    draw: params.draw,
                    records_total: peers.len(),
                    records_filtered: peers.len(),
                    data: peers,
                };

                Ok(response)
            })
            .await
            .context("Failed to read peer info")?;

        Ok(rpc_result)
    }.await;

    wrap_response(result)
}