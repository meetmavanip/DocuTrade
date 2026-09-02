use reqwest::multipart;
use crate::errors::AppError;
use std::env;

/// Uploads a document buffer to an IPFS node and returns the CID
pub async fn upload_to_ipfs(file_name: &str, content: Vec<u8>, mime_type: &str) -> Result<String, AppError> {
    let ipfs_api = env::var("IPFS_API_URL").unwrap_or_else(|_| "http://localhost:5001/api/v0".to_string());
    
    let client = reqwest::Client::new();
    
    let part = multipart::Part::bytes(content)
        .file_name(file_name.to_string())
        .mime_str(mime_type)
        .map_err(|e| AppError::Internal(format!("Failed to parse mime type: {}", e)))?;
        
    let form = multipart::Form::new().part("file", part);
    
    let response = client
        .post(&format!("{}/add", ipfs_api))
        .multipart(form)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("IPFS upload failed: {}", e)))?;
        
    if !response.status().is_success() {
        return Err(AppError::Internal("IPFS node returned error".into()));
    }
    
    // IPFS returns JSON: {"Name":"...", "Hash":"Qm...", "Size":"..."}
    let data: serde_json::Value = response.json().await
        .map_err(|e| AppError::Internal(format!("Failed to parse IPFS response: {}", e)))?;
        
    let cid = data["Hash"].as_str()
        .ok_or_else(|| AppError::Internal("Missing CID in IPFS response".into()))?;
        
    Ok(cid.to_string())
}
