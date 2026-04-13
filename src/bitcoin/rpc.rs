use crate::configuration::Network;
use crate::domain::block::BlockInfo;
use crate::AppState;
use actix_web::web;
use bitcoincore_rpc::bitcoin::Address;
use bitcoincore_rpc::json::{ImportDescriptors, Timestamp};
use bitcoincore_rpc::{Auth, Client as BitcoinClient, Client, RpcApi};
use serde_json::{json, Value};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

pub fn create_wallet_client(
    rpc_url: &str, wallet_name: &str, rpc_user: &str, rpc_password: &str,
) -> Result<Client, Box<dyn Error>> {
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
            } else if err.contains("lock") {
                println!("Wallet locked (normal), waiting 1s...");
                sleep(Duration::from_secs(1));

                if rpc.list_wallets()?.contains(&name.to_string()) {
                    println!("Now loaded");
                } else {
                    println!("Still locked but continuing - probably fine");
                }
            } else {
                return Err(e.into());
            }
        }
    }

    Ok(())
}

pub fn setup_mining_address(rpc: &BitcoinClient) -> Result<Address, Box<dyn Error>> {
    println!("\n=== Setting up mining address ===");
    let mining_address_unchecked = rpc.get_new_address(None, None)?;
    let mining_address =
        mining_address_unchecked.require_network(bitcoin::network::Network::Regtest)?;
    println!("Mining address: {}", mining_address.to_string());
    Ok(mining_address)
}

pub fn mine_until_positive_balance(
    rpc: &BitcoinClient, mining_address: &Address,
) -> Result<u32, Box<dyn Error>> {
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

    let student_wallet =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../datadir").join("student_wallet.json");
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

pub fn get_blocks_info(
    rpc: &BitcoinClient, network: Network, length: Option<u64>,
) -> Result<Vec<BlockInfo>, std::io::Error> {
    let blockchain_info = match rpc.get_blockchain_info() {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Error getting blockchain info: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to lock RPC client: {}", e),
            ));
        }
    };

    let current_height = blockchain_info.blocks;
    let length = length.unwrap_or(25);
    let start = if current_height >= length { current_height - length + 1 } else { 0 };

    let mut blocks = Vec::new();

    // TODO: Performance Optimization (Multi-threading)
    // The standard `bitcoincore-rpc` client is synchronous and blocks the thread on every call.
    // To process blocks in parallel:
    // 1. Wrap the `rpc` client in an `Arc` (e.g., `Arc<BitcoinClient>`) to clone it safely across threads.
    // 2. Use `tokio::task::spawn_blocking` inside a loop for each block height to fetch data concurrently.
    // 3. Collect and resolve all handles using `futures::future::join_all(tasks).await`.
    // Note: Ensure `rpcworkqueue` and `rpcthreads` are increased in `bitcoin.conf` to handle concurrent requests!
    for height in start..=current_height {
        let block_hash = match rpc.get_block_hash(height) {
            Ok(hash) => hash,
            Err(e) => {
                eprintln!("Error getting block hash at height {}: {}", height, e);
                continue;
            }
        };

        let block = match rpc.get_block(&block_hash) {
            Ok(block) => block,
            Err(e) => {
                eprintln!("Error getting block {}: {}", block_hash, e);
                continue;
            }
        };

        let block_stats = match rpc.get_block_stats(height) {
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
            network,
            height: height as i64,
            hash: block_hash.to_string(),
            time: block.header.time as i64,
            tx_count: block.txdata.len() as i32,
            avg_fee_sat: avg_fee_sats as i64,
            avg_feerate: avg_fee_rate_sats,
            total_fees_sat: total_fees_sats as i64,
            difficulty: block.header.difficulty_float(),
            size: block_stats.total_size as i32,
            weight: block_stats.total_weight as i32,
            subsidy_sat: block_stats.subsidy.to_sat() as i64,
            indexed_at: None,
        });
    }

    blocks.sort_by(|a, b| b.height.cmp(&a.height));

    Ok(blocks)
}

pub async fn get_last_block_height(state: &AppState) -> Result<i64, String> {
    let self1 = &state.node_manager;
    match self1.get_default_current_client().lock() {
        Ok(client) => {
            let info = client.get_blockchain_info().map_err(|e| e.to_string())?;
            Ok(info.blocks as i64)
        }
        Err(e) => Err(format!("Failed to lock RPC client: {}", e)),
    }
}

pub fn get_mempool_tx_count(rpc: &BitcoinClient) -> Result<usize, String> {
    let txids = rpc.get_raw_mempool().map_err(|e| e.to_string())?;
    Ok(txids.len())
}

pub fn get_peer_count(rpc: &BitcoinClient) -> Result<usize, String> {
    let peers = rpc.get_peer_info().map_err(|e| e.to_string())?;
    Ok(peers.len())
}

pub fn get_network_hashrate(rpc: &BitcoinClient) -> Result<f64, String> {
    let hashrate = rpc.get_network_hash_ps(None, None).map_err(|e| e.to_string())?;
    Ok(hashrate)
}

pub fn get_block_details(
   client: &Client, hash_str: String,
) -> Result<Value, bitcoincore_rpc::Error> {
    let hash = hash_str.parse().expect("invalid hash");

    let header_info = client.get_block_header_info(&hash)?;
    let block = client.get_block(&hash)?;

    let transactions: Vec<Value> = block
        .txdata
        .iter()
        .map(|tx| {
            let mut tx_obj = serde_json::Map::new();

            tx_obj.insert("txid".to_string(), json!(tx.compute_txid().to_string()));
            tx_obj.insert("version".to_string(), json!(tx.version));
            tx_obj.insert("lock_time".to_string(), json!(tx.lock_time));
            tx_obj.insert("size".to_string(), json!(tx.total_size()));
            tx_obj.insert("vsize".to_string(), json!(tx.vsize()));
            tx_obj.insert("weight".to_string(), json!(tx.weight()));

            let inputs: Vec<Value> = tx
                .input
                .iter()
                .map(|input| {
                    let mut input_obj = serde_json::Map::new();
                    input_obj
                        .insert("txid".to_string(), json!(input.previous_output.txid.to_string()));
                    input_obj.insert("vout".to_string(), json!(input.previous_output.vout));
                    input_obj.insert("sequence".to_string(), json!(input.sequence));
                    input_obj.insert("script_sig".to_string(), json!(input.script_sig.to_string()));

                    let witness = &input.witness;
                    let witness_values: Vec<String> =
                        witness.iter().map(|w| format!("{:?}", w)).collect();
                    input_obj.insert("witness".to_string(), json!(witness_values));

                    Value::Object(input_obj)
                })
                .collect();
            tx_obj.insert("inputs".to_string(), json!(inputs));

            let outputs: Vec<Value> = tx
                .output
                .iter()
                .map(|output| {
                    let mut output_obj = serde_json::Map::new();
                    output_obj.insert("value".to_string(), json!(output.value.to_btc()));
                    output_obj.insert(
                        "script_pubkey".to_string(),
                        json!(output.script_pubkey.to_string()),
                    );

                    Value::Object(output_obj)
                })
                .collect();
            tx_obj.insert("outputs".to_string(), json!(outputs));

            serde_json::Value::Object(tx_obj)
        })
        .collect();

    let block_json = json!({
        "hash": hash_str,
        "height": header_info.height,
        "version": header_info.version,
        "time": header_info.time,
        "nonce": header_info.nonce,
        "bits": header_info.bits,
        "difficulty": header_info.difficulty,
        "merkleroot": header_info.merkle_root,
        "tx_count": block.txdata.len(),
        "transactions": transactions
    });

    Ok(block_json)
}
