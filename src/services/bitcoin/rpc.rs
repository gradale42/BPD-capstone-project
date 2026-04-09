use bitcoincore_rpc::bitcoin::{Address, Network};
use bitcoincore_rpc::{Auth, Client as BitcoinClient, Client, RpcApi};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use actix_web::web;
use bitcoincore_rpc::json::{ImportDescriptors, Timestamp};
use crate::AppState;
use crate::domain::block::BlockInfo;

#[deprecated(note = "Use BitcoinNodeManager instead")]
pub fn connect() -> Result<BitcoinClient, Box<dyn std::error::Error>> {
    let bitcoin_rpc = BitcoinClient::new(
        "http://localhost:18443",
        Auth::UserPass("alice".to_string(), "password".to_string()),
    )?;

    println!("Blockchain Info: {:?}", bitcoin_rpc.get_blockchain_info()?);
    Ok(bitcoin_rpc)
}

#[deprecated(note = "Use BitcoinNodeManager instead")]
pub fn connect_to_wallet(wallet_name: &str) -> Result<BitcoinClient, Box<dyn Error>> {
    let url = format!("http://localhost:18443/wallet/{}", wallet_name);
    BitcoinClient::new(
        &url,
        Auth::UserPass("alice".to_string(), "password".to_string()),
    ).map_err(|e| e.into())
}


pub fn create_client(rpc_url: &str, rpc_user: &str, rpc_password: &str) -> Result<Client, Box<dyn Error>> {
    let auth = Auth::UserPass(rpc_user.to_string(), rpc_password.to_string());
    Ok(Client::new(rpc_url, auth)?)
}

pub fn create_wallet_client(rpc_url: &str, wallet_name: &str, rpc_user: &str, rpc_password: &str) -> Result<Client, Box<dyn Error>> {
    let wallet_url = format!("{}/wallet/{}", rpc_url, wallet_name);
    let auth = Auth::UserPass(rpc_user.to_string(), rpc_password.to_string());
    Ok(Client::new(&wallet_url, auth)?)
}

pub fn setup_wallet(rpc: &BitcoinClient, name: &str) -> Result<(), Box<dyn Error>> {
    println!("\n=== Setting up {} wallet ===", name);

    if rpc.list_wallets()?.contains(&name.to_string()) {
        println!("{} already loaded", name);
        return Ok(());
    }

    match rpc.load_wallet(name) {
        Ok(_) => println!("Loaded existing wallet"),
        Err(e) => {
            let err = e.to_string();

            if err.contains("Path does not exist") {
                println!("Creating new wallet...");
                rpc.create_wallet(name, None, None, None, None)?;

                for _ in 0..4 {
                    sleep(Duration::from_millis(500));
                    if rpc.list_wallets()?.contains(&name.to_string()) {
                        println!("Auto-loaded");
                        return Ok(());
                    }
                }

                rpc.load_wallet(name)?;
                println!("Manually loaded");
            }
            else if err.contains("lock") {
                println!("Wallet locked (normal), waiting 1s...");
                sleep(Duration::from_secs(1));

                if rpc.list_wallets()?.contains(&name.to_string()) {
                    println!("Now loaded");
                } else {
                    println!("Still locked but continuing - probably fine");
                }
            }
            else {
                return Err(e.into());
            }
        }
    }

    Ok(())
}

pub fn setup_mining_address(rpc: &BitcoinClient) -> Result<Address, Box<dyn Error>> {
    println!("\n=== Setting up mining address ===");
    let mining_address_unchecked = rpc.get_new_address(None, None)?;
    let mining_address = mining_address_unchecked.require_network(Network::Regtest)?;
    println!("Mining address: {}", mining_address.to_string());
    Ok(mining_address)
}

pub fn mine_until_positive_balance(rpc: &BitcoinClient, mining_address: &Address) -> Result<u32, Box<dyn Error>> {
    println!("\n=== Mining blocks until positive balance ===");
    let mut blocks_mined = 0;
    let mut balance = 0.0;

    while balance <= 0.0 {
        let _block_hashes = rpc.generate_to_address(1, mining_address)?;
        blocks_mined += 1;

        balance = rpc.get_balance(None, None)?.to_btc();

        println!("Mined block #{}, current balance: {} BTC", blocks_mined, balance);
        sleep(Duration::from_millis(500));
    }

    println!("Positive balance achieved after {} blocks!", blocks_mined);
    println!("Final balance: {} BTC", balance);
    println!("Total blocks mined: {}", blocks_mined);

    Ok(blocks_mined)
}

pub fn import_descriptors(rpc: &BitcoinClient) -> Result<(), Box<dyn Error>> {
    println!("\n=== Importing test wallets ===");

    setup_wallet(&rpc, "student")?;

    let student_wallet = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("datadir")
        .join("student_wallet.json");
    print!("{:?}", student_wallet);
    let student_wallet_data = fs::read_to_string(student_wallet)?;
    print!("{}", student_wallet_data);

    let descriptor = ImportDescriptors {
        descriptor: student_wallet_data.trim().to_string(),
        timestamp: Timestamp::Now,
        active: Some(true),
        range: None,
        next_index: None,
        internal: Some(false),
        label: None,
    };

    rpc.import_descriptors(descriptor)?;

    Ok(())
}

pub async fn get_blocks_info(
    state: &AppState,
    length: Option<u64>,
) -> Result<Vec<BlockInfo>, std::io::Error> {
    match state.get_default_bitcoin_client().lock() {
        Ok(client) => {

            let blockchain_info = match client.get_blockchain_info() {
                Ok(info) => info,
                Err(e) => {
                    eprintln!("Error getting blockchain info: {}", e);
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to lock RPC client: {}", e)));
                }
            };

            let current_height = blockchain_info.blocks;
            let length = length.unwrap_or(25);
            let start = if current_height >= length {
                current_height - length + 1
            } else {
                0
            };

            let mut blocks = Vec::new();

            for height in start..=current_height {

                let block_hash = match client.get_block_hash(height) {
                    Ok(hash) => hash,
                    Err(e) => {
                        eprintln!("Error getting block hash at height {}: {}", height, e);
                        continue;
                    }
                };

                let block = match client.get_block(&block_hash) {
                    Ok(block) => block,
                    Err(e) => {
                        eprintln!("Error getting block {}: {}", block_hash, e);
                        continue;
                    }
                };

                let block_stats = match client.get_block_stats(height) {
                    Ok(stats) => stats,
                    Err(e) => {
                        eprintln!("Error getting block stats for height {}: {}", height, e);
                        continue;
                    }
                };

                let avg_fee_sats = block_stats.avg_fee.to_sat() as f64;
                let avg_fee_rate_sats = block_stats.avg_fee_rate.to_sat() as f64;
                let total_fees_sats = block_stats.total_fee.to_sat() as f64;

                blocks.push(BlockInfo {
                    height: height as i64,
                    hash: block_hash.to_string(),
                    time: block.header.time as i64,
                    tx_count: block.txdata.len() as i32,
                    avg_fee_sat: avg_fee_sats as i64,
                    avg_feerate: avg_fee_rate_sats,
                    total_fees_sat: total_fees_sats as i64,
                    difficulty: 0.0, //block.header.difficulty() as f64,
                    size: 0, // block size is not directly available in the block data, you may need to calculate it or fetch it separately
                    weight: 0, // block weight is not directly available in the block data, you may need to calculate it or fetch it separately
                    subsidy_sat: block_stats.subsidy.to_sat() as i64,
                    indexed_at: None, // you can set this to the current timestamp when you insert it into the database
                });
            }

            blocks.sort_by(|a, b| b.height.cmp(&a.height));

            Ok(blocks)
        }
        Err(e) => {
            eprintln!("Error locking RPC client: {}", e);
            Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to lock RPC client: {}", e)))
        }
    }
}

pub async fn get_last_block_height(state: &AppState) -> Result<i64, String> {
    match state.get_default_bitcoin_client().lock() {
        Ok(client) => {
            let info = client.get_blockchain_info().map_err(|e| e.to_string())?;
            Ok(info.blocks as i64)
        }
        Err(e) => Err(format!("Failed to lock RPC client: {}", e)),
    }
}

pub async fn get_mempool_tx_count(state: &AppState) -> Result<usize, String> {
    match state.get_default_bitcoin_client().lock() {
        Ok(client) => {
            let txids = client.get_raw_mempool().map_err(|e| e.to_string())?;
            Ok(txids.len())
        }
        Err(e) => Err(format!("Failed to lock RPC client: {}", e)),
    }
}

pub async fn get_peer_count(state: &AppState) -> Result<usize, String> {
    match state.get_default_bitcoin_client().lock() {
        Ok(client) => {
            let peers = client.get_peer_info().map_err(|e| e.to_string())?;
            Ok(peers.len())
        }
        Err(e) => Err(format!("Failed to lock RPC client: {}", e)),
    }
}

pub async fn get_network_hashrate(state: &AppState) -> Result<f64, String> {
    match state.get_default_bitcoin_client().lock() {
        Ok(client) => {
            let hashrate = client.get_network_hash_ps(None, None).map_err(|e| e.to_string())?;
            Ok(hashrate as f64 / 1e18) // EH/s
        }
        Err(e) => Err(format!("Failed to lock RPC client: {}", e)),
    }
}





