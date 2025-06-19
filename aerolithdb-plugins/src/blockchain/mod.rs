//! Blockchain integration module for cryptocurrency payments
//! 
//! This module provides abstract interfaces and concrete implementations
//! for interacting with various blockchain networks.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod tron;
pub mod solana;

/// Generic blockchain interface for payment operations
#[async_trait]
pub trait BlockchainProvider: Send + Sync {
    /// Get provider name and network information
    fn get_info(&self) -> BlockchainInfo;
    
    /// Initialize connection to the blockchain network
    async fn initialize(&mut self, config: BlockchainConfig) -> Result<()>;
    
    /// Validate a wallet address format
    fn validate_address(&self, address: &str) -> Result<bool>;
    
    /// Get native token balance for an address
    async fn get_native_balance(&self, address: &str) -> Result<u64>;
    
    /// Get token balance for a specific token contract
    async fn get_token_balance(&self, address: &str, token_contract: &str) -> Result<u64>;
    
    /// Create a transaction for token transfer
    async fn create_transaction(&self, request: TransactionRequest) -> Result<UnsignedTransaction>;
    
    /// Broadcast a signed transaction to the network
    async fn broadcast_transaction(&self, signed_tx: &str) -> Result<String>;
    
    /// Get transaction status and confirmations
    async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus>;
    
    /// Estimate transaction fees
    async fn estimate_fees(&self, request: &TransactionRequest) -> Result<FeeEstimate>;
    
    /// Get current network status
    async fn get_network_status(&self) -> Result<NetworkStatus>;
}

/// Blockchain provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainInfo {
    pub name: String,
    pub network: String,
    pub chain_id: Option<u64>,
    pub native_token: String,
    pub supported_tokens: Vec<TokenInfo>,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub symbol: String,
    pub name: String,
    pub contract_address: String,
    pub decimals: u8,
}

/// Blockchain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub rpc_endpoints: Vec<String>,
    pub network_id: String,
    pub api_keys: std::collections::HashMap<String, String>,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
}

/// Transaction request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub from_address: String,
    pub to_address: String,
    pub amount: u64,
    pub token_contract: Option<String>, // None for native token
    pub gas_limit: Option<u64>,
    pub gas_price: Option<u64>,
    pub memo: Option<String>,
}

/// Unsigned transaction ready for signing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedTransaction {
    pub transaction_data: String,
    pub hash_to_sign: String,
    pub estimated_fee: u64,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Transaction status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    pub hash: String,
    pub status: TxStatus,
    pub confirmations: u32,
    pub block_number: Option<u64>,
    pub gas_used: Option<u64>,
    pub gas_price: Option<u64>,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// Transaction status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TxStatus {
    Pending,
    Confirmed,
    Failed,
    Reverted,
}

/// Fee estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimate {
    pub gas_limit: u64,
    pub gas_price: u64,
    pub total_fee: u64,
    pub estimated_confirmation_time: u32, // seconds
}

/// Network status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub latest_block: u64,
    pub network_congestion: CongestionLevel,
    pub average_block_time: u32, // seconds
    pub recommended_gas_price: u64,
}

/// Network congestion level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CongestionLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Blockchain manager for coordinating multiple providers
pub struct BlockchainManager {
    providers: std::collections::HashMap<String, Box<dyn BlockchainProvider>>,
}

impl BlockchainManager {
    /// Create a new blockchain manager
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
        }
    }
    
    /// Register a blockchain provider
    pub async fn register_provider(&mut self, name: String, mut provider: Box<dyn BlockchainProvider>) -> Result<()> {
        // Note: In a real implementation, you'd pass the appropriate config here
        let config = BlockchainConfig {
            rpc_endpoints: vec!["http://localhost:8545".to_string()],
            network_id: "testnet".to_string(),
            api_keys: std::collections::HashMap::new(),
            timeout_seconds: 30,
            retry_attempts: 3,
        };
        
        provider.initialize(config).await?;
        self.providers.insert(name, provider);
        Ok(())
    }
    
    /// Get a blockchain provider by name
    pub fn get_provider(&self, name: &str) -> Option<&dyn BlockchainProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }
    
    /// List all available providers
    pub fn list_providers(&self) -> Vec<BlockchainInfo> {
        self.providers.values()
            .map(|provider| provider.get_info())
            .collect()
    }
}

impl Default for BlockchainManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blockchain_manager_creation() {
        let manager = BlockchainManager::new();
        assert_eq!(manager.providers.len(), 0);
    }
}
