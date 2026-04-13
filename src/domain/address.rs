#[derive(Debug, serde::Serialize)]
pub struct AddressInfo {
    pub address: String,
    pub balance: f64,
    pub label: Option<String>,
    pub received: f64,
    pub spent: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct DescriptorInfo {
    pub descriptor: String,
    pub active: bool,
    pub range: Option<Vec<u64>>,
    pub timestamp: u64,
}