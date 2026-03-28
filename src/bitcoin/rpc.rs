use bitcoincore_rpc::bitcoin::{Address, Network};
use bitcoincore_rpc::{Auth, Client as BitcoinClient, RpcApi};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use bitcoincore_rpc::json::{ImportDescriptors, Timestamp};

pub fn connect() -> Result<BitcoinClient, Box<dyn std::error::Error>> {
    let bitcoin_rpc = BitcoinClient::new(
        "http://localhost:18443",
        Auth::UserPass("alice".to_string(), "password".to_string()),
    )?;

    println!("Blockchain Info: {:?}", bitcoin_rpc.get_blockchain_info()?);
    Ok(bitcoin_rpc)
}

pub fn connect_to_wallet(wallet_name: &str) -> Result<BitcoinClient, Box<dyn Error>> {
    let url = format!("http://localhost:18443/wallet/{}", wallet_name);
    BitcoinClient::new(
        &url,
        Auth::UserPass("alice".to_string(), "password".to_string()),
    ).map_err(|e| e.into())
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






