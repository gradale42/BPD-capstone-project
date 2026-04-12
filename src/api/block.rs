use actix_web::{web, HttpResponse, Responder};
use bitcoincore_rpc::RpcApi;
use serde_json::{json, Value};
use std::fs;
use crate::AppState;
use bitcoincore_rpc::bitcoin::Witness;
use crate::bitcoin::execute_rpc;

pub async fn get_block_by_hash(
    state: web::Data<AppState>,
    block_hash: web::Path<String>,
) -> impl Responder {
    let hash_str = block_hash.into_inner();

    execute_rpc(&state.node_manager, "", move |client| {

        let hash = hash_str.parse().expect("invalid hash");

        let header_info = client.get_block_header_info(&hash)?;
        let block = client.get_block(&hash)?;

        let transactions: Vec<Value> = block.txdata
            .iter()
            .map(|tx| {
                let mut tx_obj = serde_json::Map::new();

                tx_obj.insert("txid".to_string(), json!(tx.compute_txid().to_string()));
                tx_obj.insert("version".to_string(), json!(tx.version));
                tx_obj.insert("lock_time".to_string(), json!(tx.lock_time));
                tx_obj.insert("size".to_string(), json!(tx.total_size()));
                tx_obj.insert("vsize".to_string(), json!(tx.vsize()));
                tx_obj.insert("weight".to_string(), json!(tx.weight()));

                let inputs: Vec<Value> = tx.input.iter()
                    .map(|input| {
                        let mut input_obj = serde_json::Map::new();
                        input_obj.insert("txid".to_string(), json!(input.previous_output.txid.to_string()));
                        input_obj.insert("vout".to_string(), json!(input.previous_output.vout));
                        input_obj.insert("sequence".to_string(), json!(input.sequence));
                        input_obj.insert("script_sig".to_string(), json!(input.script_sig.to_string()));

              /*          if let Some(witness) = &input.witness {
                            let witness_values: Vec<String> = witness.iter()
                                .map(|w| format!("{:?}", w))
                                .collect();
                            input_obj.insert("witness".to_string(), json!(witness_values));
                        }*/

                        serde_json::Value::Object(input_obj)
                    })
                    .collect();
                tx_obj.insert("inputs".to_string(), json!(inputs));

                let outputs: Vec<Value> = tx.output.iter()
                    .map(|output| {
                        let mut output_obj = serde_json::Map::new();
                        output_obj.insert("value".to_string(), json!(output.value.to_btc()));
                        output_obj.insert("script_pubkey".to_string(), json!(output.script_pubkey.to_string()));

                        serde_json::Value::Object(output_obj)
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
    }).await
}

pub async fn get_block_by_hash1(
    state: web::Data<AppState>,
    block_hash: web::Path<String>,
) -> impl Responder {
    let self1 = &state.node_manager;
    match self1.get_default_current_client().lock() {
        Ok(client) => {

            let hash = match block_hash.parse() {
                Ok(h) => h,
                Err(e) => {
                    return HttpResponse::BadRequest().json(json!({
                        "status": "error",
                        "message": format!("Invalid block hash: {}", e)
                    }));
                }
            };

            match client.get_block_header_info(&hash) {
                Ok(header_info) => {
                    match client.get_block(&hash) {
                        Ok(block) => {

                            let transactions: Vec<Value> = block.txdata
                                .iter()
                                .map(|tx| {
                                    let mut tx_obj = serde_json::Map::new();

                                    tx_obj.insert("txid".to_string(), json!(tx.compute_txid().to_string()));
                                    tx_obj.insert("version".to_string(), json!(tx.version));
                                    tx_obj.insert("lock_time".to_string(), json!(tx.lock_time));
                                    tx_obj.insert("size".to_string(), json!(tx.total_size()));
                                    tx_obj.insert("vsize".to_string(), json!(tx.vsize()));
                                    tx_obj.insert("weight".to_string(), json!(tx.weight()));

                                    let inputs: Vec<Value> = tx.input.iter()
                                        .map(|input| {
                                            let mut input_obj = serde_json::Map::new();
                                            input_obj.insert("txid".to_string(), json!(input.previous_output.txid.to_string()));
                                            input_obj.insert("vout".to_string(), json!(input.previous_output.vout));
                                            input_obj.insert("sequence".to_string(), json!(input.sequence));

                                            todo!("decode witness");
                            /*                if let Some(script_sig) = &input.script_sig {
                                                input_obj.insert("script_sig".to_string(), json!(script_sig.to_string()));
                                            }

                                            match &input.witness {
                                                Some(witness) => {
                                                    let witness_values: Vec<String> = witness.iter()
                                                        .map(|w| format!("{:?}", w))
                                                        .collect();
                                                    input_obj.insert("witness".to_string(), json!(witness_values));
                                                }
                                                None => {}
                                            }
*/
                                            serde_json::Value::Object(input_obj)
                                        })
                                        .collect();
                                    tx_obj.insert("inputs".to_string(), json!(inputs));

                                    let outputs: Vec<Value> = tx.output.iter()
                                        .map(|output| {
                                            let mut output_obj = serde_json::Map::new();
                                            output_obj.insert("value".to_string(), json!(output.value.to_btc()));
                                            output_obj.insert("script_pubkey".to_string(), json!(output.script_pubkey.to_string()));

                                            todo!("decode address");
                                           /*
                                            if let Some(address) = &output.script_pubkey.address {
                                                output_obj.insert("address".to_string(), json!(address.to_string()));
                                            } */

                                            serde_json::Value::Object(output_obj)
                                        })
                                        .collect();
                                    tx_obj.insert("outputs".to_string(), json!(outputs));

                                    serde_json::Value::Object(tx_obj)
                                })
                                .collect();

                            let block_json = json!({
                                "hash": hash.to_string(),
                                "height": header_info.height,
                                "version": header_info.version,
                                "time": header_info.time,
                              //  "mediantime": header_info.mediantime,
                                "nonce": header_info.nonce,
                                "bits": header_info.bits,
                                "difficulty": header_info.difficulty,
                                "merkleroot": header_info.merkle_root,
                                "tx_count": block.txdata.len(),
                                //"size": header_info.size,
                             //   "weight": header_info.weight,
                                "transactions": transactions
                            });

                            HttpResponse::Ok().json(json!({
                                "status": "success",
                                "block": block_json
                            }))
                        }
                        Err(e) => HttpResponse::NotFound().json(json!({
                            "status": "error",
                            "message": format!("Block data not found: {}", e)
                        }))
                    }
                }
                Err(e) => HttpResponse::NotFound().json(json!({
                    "status": "error",
                    "message": format!("Block header not found: {}", e)
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Failed to lock RPC client: {}", e)
        }))
    }
}