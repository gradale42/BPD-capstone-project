use crate::configuration::{BitcoinNodeConfig, Network};
use bitcoincore_rpc::{Auth, Client};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientKey {
    pub network: Network,
    pub wallet_name: String,
}

impl ClientKey {
    pub fn new(network: Network, wallet_name: String) -> Self {
        Self { network, wallet_name }
    }

    pub fn default_for_network(network: Network) -> Self {
        Self::new(network, String::new())
    }
}

pub struct BitcoinNodeManager {
    clients: DashMap<ClientKey, Arc<Mutex<Client>>>,
    network_configs: HashMap<Network, BitcoinNodeConfig>,
    current_network: Arc<Mutex<Network>>,
}

impl BitcoinNodeManager {
    pub fn new(network_configs: HashMap<Network, BitcoinNodeConfig>, default_network: Network) -> Self {
        Self {
            clients: DashMap::new(),
            network_configs,
            current_network: Arc::new(Mutex::new(default_network)),
        }
    }

    pub fn set_current_network(&self, network: Network) {
        let mut current = self.current_network.lock().unwrap();
        *current = network;
        println!("🔄 Switched to network: {}", network);
    }

    pub fn get_current_network(&self) -> Network {
        *self.current_network.lock().unwrap()
    }

    pub fn get_current_client(&self, wallet_name: &str) -> Arc<Mutex<Client>> {
        let network = self.get_current_network();
        let key = ClientKey::new(network, wallet_name.to_string());

        self.clients.entry(key.clone()).or_insert_with(|| {
            println!("=== Creating Bitcoin client for network: {}, wallet: {} ===",
                     network, if wallet_name.is_empty() { "default" } else { wallet_name });

            let config = self.network_configs.get(&network).expect(&format!(
                "No configuration found for network: {}", network
            ));

            let wallet_url = if wallet_name.is_empty() {
                config.rpc_url.clone()
            } else {
                format!("{}/wallet/{}", config.rpc_url, wallet_name)
            };

            let auth = Auth::UserPass(config.rpc_user.clone(), config.rpc_password.clone());

            let client = Client::new(&wallet_url, auth)
                .expect(&format!("Failed to create Bitcoin RPC client for network {} wallet {}",
                                 network, wallet_name));

            Arc::new(Mutex::new(client))
        }).value().clone()
    }

    pub fn get_default_current_client(&self) -> Arc<Mutex<Client>> {
        self.get_current_client("")
    }

    pub async fn execute_rpc<F, R>(&self, wallet_name: &str, f: F) -> Result<R, String>
    where
        F: FnOnce(&bitcoincore_rpc::Client) -> Result<R, bitcoincore_rpc::Error> + Send + 'static,
        R: serde::Serialize + Send + 'static,
    {
        let client_arc = self.get_current_client(wallet_name);

        let result = tokio::task::spawn_blocking(move || {
            let client_guard = client_arc.lock().map_err(|_| "Lock poisoning error".to_string())?;
            f(&*client_guard).map_err(|e| e.to_string())
        }).await;

        match result {
            Ok(Ok(data)) => Ok(data),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(format!("RPC error: {}", e)),
        }
    }

}