use serde::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub server_port: u16,
    pub arbitrum_rpc_url: String,
    pub private_key: String,
    pub contract_address: String,
    pub ipfs_gateway: String,
    pub document_verification_contract: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://docutrade:docutrade@localhost:5432/docutrade".to_string());
        
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "super_secret_jwt_key_for_dev_only".to_string());
            
        let server_port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .unwrap_or(3000);
            
        let arbitrum_rpc_url = env::var("ARBITRUM_RPC_URL")
            .unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string());
            
        let private_key = env::var("PRIVATE_KEY")
            .unwrap_or_else(|_| "0000000000000000000000000000000000000000000000000000000000000000".to_string());
            
        let contract_address = env::var("CONTRACT_ADDRESS")
            .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());
            
        let ipfs_gateway = env::var("IPFS_GATEWAY")
            .unwrap_or_else(|_| "http://localhost:5001".to_string());

        let document_verification_contract = env::var("DOCUMENT_VERIFICATION_CONTRACT")
            .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());

        Self {
            database_url,
            jwt_secret,
            server_port,
            arbitrum_rpc_url,
            private_key,
            contract_address,
            ipfs_gateway,
            document_verification_contract,
        }
    }
}
