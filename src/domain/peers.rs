use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct PeerInfo {
    pub peer_id: u64,
    pub inbound: bool,
    pub subver: String,
    pub version: u64,
    pub bytes_sent_total: u64,
    pub bytes_recv_total: u64,
    pub bytes_sent_delta: u64,
    pub bytes_recv_delta: u64,
    pub ping: f64,
}
