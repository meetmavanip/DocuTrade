use sha2::{Sha256, Digest};
use crate::errors::AppError;

/// Calculate SHA-256 hash of document bytes
pub fn hash_document(content: &[u8]) -> Result<String, AppError> {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
