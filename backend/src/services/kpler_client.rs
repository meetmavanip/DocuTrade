use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use anyhow::{Result, Context};
use tracing::{info, error};

#[derive(Debug, Clone)]
pub struct KplerClient {
    client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TradeVessel {
    pub name: Option<String>,
    pub imo: Option<i32>,
    pub mmsi: Option<i32>,
    pub id: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Port {
    pub name: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KplerTrade {
    pub vessel: Option<TradeVessel>,
    pub origin: Option<Port>,
    pub destination: Option<Port>,
    pub eta: Option<String>,
    pub status: Option<String>,
    pub voyage_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KplerPortCall {
    pub vessel: Option<TradeVessel>,
    pub port_name: Option<String>,
    pub location: Option<String>,
    pub eta: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub status: Option<String>,
    pub voyage_id: Option<String>,
}

impl KplerClient {
    pub fn new() -> Result<Self> {
        let base_url = env::var("KPLER_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.kpler.com/v2/cargo".to_string());
        
        let api_key = env::var("KPLER_API_KEY")
            .unwrap_or_else(|_| "TEST_KEY".to_string()); // Fallback for testing/dev
            
        Ok(Self {
            client: Client::new(),
            base_url,
            api_key,
        })
    }
    
    pub async fn get_trades_by_vessel(&self, vessel_identifier: &str) -> Result<Vec<KplerTrade>> {
        let url = format!("{}/trades", self.base_url);
        
        let res = self.client.get(&url)
            .bearer_auth(&self.api_key)
            .query(&[("vessels", vessel_identifier)])
            .send()
            .await?;
            
        if !res.status().is_success() {
            error!("Kpler API error for trades: {}", res.status());
            return Ok(vec![]);
        }
            
        // For robustness, parse flexibly as Kpler response structure can vary.
        let bytes = res.bytes().await?;
        if let Ok(response) = serde_json::from_slice::<Vec<KplerTrade>>(&bytes) {
            return Ok(response);
        }
        
        #[derive(Deserialize)]
        struct Wrapper {
            content: Option<Vec<KplerTrade>>
        }
        
        if let Ok(wrapper) = serde_json::from_slice::<Wrapper>(&bytes) {
            return Ok(wrapper.content.unwrap_or_default());
        }
        
        Ok(vec![])
    }
    
    pub async fn get_port_calls(&self, vessel_identifier: &str) -> Result<Vec<KplerPortCall>> {
        let url = format!("{}/port-calls", self.base_url);
        
        let res = self.client.get(&url)
            .bearer_auth(&self.api_key)
            .query(&[("vessels", vessel_identifier)])
            .send()
            .await?;
            
        if !res.status().is_success() {
            error!("Kpler API error for port calls: {}", res.status());
            return Ok(vec![]);
        }
            
        let bytes = res.bytes().await?;
        if let Ok(response) = serde_json::from_slice::<Vec<KplerPortCall>>(&bytes) {
            return Ok(response);
        }
        
        #[derive(Deserialize)]
        struct Wrapper {
            content: Option<Vec<KplerPortCall>>
        }
        
        if let Ok(wrapper) = serde_json::from_slice::<Wrapper>(&bytes) {
            return Ok(wrapper.content.unwrap_or_default());
        }
        
        Ok(vec![])
    }
}
