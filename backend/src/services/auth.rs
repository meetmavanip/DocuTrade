use crate::errors::AppError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use alloy::signers::Signature;
use alloy::primitives::Address;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash format: {}", e)))?;
    let is_valid = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();
    Ok(is_valid)
}

pub fn create_jwt(user_id: &str, role: &str) -> Result<String, AppError> {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + 24 * 3600; // 24 hours

    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp,
    };

    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "super_secret_jwt_key_for_dev_only".into());
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ).map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))
}

pub fn generate_nonce() -> String {
    let uuid = Uuid::new_v4().to_string();
    format!("DOCUTRADE-AUTH-{}", uuid)
}

pub fn verify_signature(wallet_address: &str, nonce: &str, signature: &str) -> Result<bool, AppError> {
    // Attempt to parse the signature
    let sig = Signature::from_str(signature)
        .map_err(|_| AppError::Auth("Invalid signature format".into()))?;
    
    // Alloy's recover_address_from_msg automatically prepends the EIP-191 prefix
    let recovered_address = sig.recover_address_from_msg(nonce)
        .map_err(|_| AppError::Auth("Failed to recover address from signature".into()))?;
    
    let expected_address = Address::from_str(wallet_address)
        .map_err(|_| AppError::Auth("Invalid wallet address format".into()))?;

    // Compare recovered address with the expected address (case-insensitive in string form, but Address comparison handles this)
    Ok(recovered_address == expected_address)
}

pub fn verify_jwt(token: &str) -> Result<Claims, AppError> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "super_secret_jwt_key_for_dev_only".into());
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ).map_err(|e| AppError::Auth(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims)
}
