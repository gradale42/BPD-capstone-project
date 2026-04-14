use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub bitcoin: BitcoinSettings,
    pub application_port: u16,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new("configuration.yaml", config::FileFormat::Yaml))
        .build()?;
    settings.try_deserialize::<Settings>()
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    pub fn connection_string_without_db(&self) -> String {
        format!("postgres://{}:{}@{}:{}", self.username, self.password, self.host, self.port)
    }
}


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "bitcoin_network", rename_all = "lowercase")]
pub enum Network {
    Regtest,
    Signet,
    Testnet,
    Mainnet,
}

pub const ALL_BITCOIN_NETWORKS: [Network; 4] =
    [Network::Regtest, Network::Signet, Network::Testnet, Network::Mainnet];

impl Network {
    pub fn rpc_port(&self) -> u16 {
        match self {
            Network::Regtest => 18443,
            Network::Signet => 38332,
            Network::Testnet => 18332,
            Network::Mainnet => 8332,
        }
    }

    pub fn p2p_port(&self) -> u16 {
        match self {
            Network::Regtest => 18444,
            Network::Signet => 38333,
            Network::Testnet => 18333,
            Network::Mainnet => 8333,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Network::Regtest => "regtest",
            Network::Signet => "signet",
            Network::Testnet => "testnet",
            Network::Mainnet => "mainnet",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "regtest" => Some(Network::Regtest),
            "signet" => Some(Network::Signet),
            "testnet" => Some(Network::Testnet),
            "mainnet" => Some(Network::Mainnet),
            _ => None,
        }
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct BitcoinNodeConfig {
    pub rpc_url: String,
    pub rpc_user: String,
    pub rpc_password: String,
}

#[derive(serde::Deserialize)]
pub struct BitcoinSettings {
    #[serde(flatten)]
    pub default_config: BitcoinNodeConfig,
    pub networks: HashMap<String, BitcoinNodeConfig>,
}

impl BitcoinSettings {
    pub fn get_node_config(&self, network: Network) -> BitcoinNodeConfig {
        let network_str = network.as_str();

        if let Some(network_config) = self.networks.get(network_str) {
            network_config.clone()
        } else {
            // Fallback to default with port substitution
            let mut config = self.default_config.clone();
            config.rpc_url = format!("http://localhost:{}", network.rpc_port());
            config
        }
    }
}
