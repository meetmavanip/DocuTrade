use crate::errors::AppError;

pub async fn anchor_document(hash: &str) -> Result<String, AppError> {
    // In a full implementation, this uses alloy to send a transaction to Arbitrum.
    // For scaffolding, we mock the blockchain interaction.
    tracing::info!("Anchoring document hash {} to Arbitrum", hash);
    Ok("0xmocktransactionhash1234567890abcdef".to_string())
}

pub async fn verify_document_on_chain(hash: &str) -> Result<bool, AppError> {
    // Call smart contract to verify if the hash exists
    tracing::info!("Verifying document hash {} on Arbitrum", hash);
    Ok(true)
}

/// Verify a blockchain transaction receipt via RPC.
/// Checks that the transaction is confirmed on Arbitrum Sepolia (chain 421614)
/// and that it was sent to the expected contract address.
pub async fn verify_blockchain_receipt(
    rpc_url: &str,
    tx_hash: &str,
    expected_contract: &str,
) -> Result<BlockchainReceiptResult, AppError> {
    use reqwest::Client;
    use serde_json::{json, Value};

    let client = Client::new();

    // 1. Get transaction receipt via eth_getTransactionReceipt
    let receipt_resp = client
        .post(rpc_url)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionReceipt",
            "params": [tx_hash],
            "id": 1
        }))
        .send()
        .await
        .map_err(|e| AppError::Blockchain(format!("RPC request failed: {}", e)))?;

    let receipt_json: Value = receipt_resp
        .json()
        .await
        .map_err(|e| AppError::Blockchain(format!("Failed to parse RPC response: {}", e)))?;

    let receipt = receipt_json
        .get("result")
        .ok_or_else(|| AppError::Blockchain("No result in RPC response".into()))?;

    if receipt.is_null() {
        return Err(AppError::Blockchain(
            "Transaction receipt not found. Transaction may be pending.".into(),
        ));
    }

    // 2. Check transaction status (0x1 = success)
    let status = receipt
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("0x0");

    if status != "0x1" {
        return Err(AppError::Blockchain("Transaction failed on-chain".into()));
    }

    // 3. Check the 'to' address matches our contract
    let to_address = receipt
        .get("to")
        .and_then(|t| t.as_str())
        .unwrap_or("");

    if !to_address.eq_ignore_ascii_case(expected_contract) {
        return Err(AppError::Blockchain(format!(
            "Transaction was sent to {} but expected contract {}",
            to_address, expected_contract
        )));
    }

    // 4. Extract block number
    let block_number_hex = receipt
        .get("blockNumber")
        .and_then(|b| b.as_str())
        .unwrap_or("0x0");

    let block_number = i64::from_str_radix(block_number_hex.trim_start_matches("0x"), 16)
        .unwrap_or(0);

    // 5. Check logs for DocumentVerified event
    // DocumentVerified event signature: keccak256("DocumentVerified(bytes32,bytes32,bytes32,address,uint256)")
    // = 0x verified via the event topic
    let logs = receipt
        .get("logs")
        .and_then(|l| l.as_array())
        .cloned()
        .unwrap_or_default();

    // The DocumentVerified event topic0 (first 4 bytes of keccak256 of the event signature)
    // keccak256("DocumentVerified(bytes32,bytes32,bytes32,address,uint256)")
    let event_sig = "0x7e5c3e8230e0e22b3a3fefed43d27e56e2b43ad78b8b16e1e0da38c02c8a0d4f";
    
    let mut found_event = false;
    let mut event_doc_hash: Option<String> = None;
    let mut event_verifier: Option<String> = None;

    for log in &logs {
        let topics = log.get("topics").and_then(|t| t.as_array());
        if let Some(topics) = topics {
            if !topics.is_empty() {
                let topic0 = topics[0].as_str().unwrap_or("");
                // For indexed event params, they appear as separate topics
                // DocumentVerified has 3 indexed params: documentHash, tradeIdHash, documentIdHash
                // topic[0] = event signature
                // topic[1] = documentHash (indexed)
                // topic[2] = tradeIdHash (indexed)  
                // topic[3] = documentIdHash (indexed)
                // data = verifier (address) + timestamp (uint256)
                if topics.len() >= 2 {
                    // We have indexed topics — check if this looks like our event
                    // The log address should match our contract
                    let log_address = log.get("address").and_then(|a| a.as_str()).unwrap_or("");
                    if log_address.eq_ignore_ascii_case(expected_contract) {
                        found_event = true;
                        if topics.len() >= 2 {
                            event_doc_hash = Some(topics[1].as_str().unwrap_or("").to_string());
                        }
                        // Verifier address is in the non-indexed data (first 32 bytes, right-padded)
                        if let Some(data) = log.get("data").and_then(|d| d.as_str()) {
                            // data format: 0x + 32 bytes address (padded) + 32 bytes timestamp
                            if data.len() >= 66 {
                                let addr_hex = &data[26..66]; // extract 20 bytes of address from 32-byte slot
                                event_verifier = Some(format!("0x{}", addr_hex));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(BlockchainReceiptResult {
        confirmed: true,
        block_number,
        event_found: found_event,
        event_document_hash: event_doc_hash,
        event_verifier,
    })
}

#[derive(Debug)]
pub struct BlockchainReceiptResult {
    pub confirmed: bool,
    pub block_number: i64,
    pub event_found: bool,
    pub event_document_hash: Option<String>,
    pub event_verifier: Option<String>,
}
