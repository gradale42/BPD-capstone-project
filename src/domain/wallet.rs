use crate::domain::address::AddressInfo;

#[derive(Debug, serde::Serialize)]
pub struct WalletInfo {
    pub name: String,
    pub balance: f64,
    pub address_count: usize,
    pub addresses: Vec<AddressInfo>,
}

impl WalletInfo {
    pub(crate) fn default_with_name(name: String) -> Self {
        Self {
            name,
            balance: 0.0,
            address_count: 0,
            addresses: vec![],
        }
    }
}