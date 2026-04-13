//#![allow(unused)]

use bitcoin_dashboard::configuration::get_configuration;
use bitcoin_dashboard::run;
use sqlx::{PgPool, Pool, Postgres};
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection_pool = init_database().await?;
    let listener = init_api_listener()?;

    println!("\n🚀 SERVER STARTING...");
    run(listener, connection_pool)?.await?;

    Ok(())
}

async fn init_database() -> Result<Pool<Postgres> , Box<dyn std::error::Error>> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let db_url = configuration.database.connection_string();
    let connection_pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to Postgres.");
    Ok(connection_pool)
}

fn init_api_listener() -> Result<TcpListener, Box<dyn std::error::Error>> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    Ok(listener)
}