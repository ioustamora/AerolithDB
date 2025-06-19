//! Solana blockchain provider for USDC (SPL Token) payments
//! 
//! This module implements the BlockchainProvider trait for the Solana network,
//! providing support for SOL and USDC (SPL Token) transactions.

use super::*;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use dashmap::DashMap;
use std::sync::Arc;

/// Solana blockchain provider implementation
#[derive(Debug)]
pub struct SolanaProvider {
    config: Option<BlockchainConfig>,
    rpc_client: Option<SolanaRpcClient>,
    network_info: BlockchainInfo,
}

impl SolanaProvider {
    /// Create a new Solana provider
    pub fn new() -> Self {
        Self {
            config: None,
            rpc_client: None,
            network_info: Self::default_network_info(),
        }
    }
    
    /// Get default network information for Solana
    fn default_network_info() -> BlockchainInfo {
        BlockchainInfo {
            name: "Solana".to_string(),
            network: "mainnet-beta".to_string(),
            chain_id: None, // Solana doesn't use chain IDs like Ethereum
            native_token: "SOL".to_string(),
            supported_tokens: vec![
                TokenInfo {
                    symbol: "USDC".to_string(),
                    name: "USD Coin".to_string(),
                    contract_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC SPL Token
                    decimals: 6,
                },
                TokenInfo {
                    symbol: "USDT".to_string(),
                    name: "Tether USD".to_string(),
                    contract_address: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(), // USDT SPL Token
                    decimals: 6,
                },
            ],
        }
    }
    
    /// Convert SOL to lamports (smallest unit)
    fn sol_to_lamports(sol: f64) -> u64 {
        (sol * 1_000_000_000.0) as u64
    }
    
    /// Convert lamports to SOL
    fn lamports_to_sol(lamports: u64) -> f64 {
        lamports as f64 / 1_000_000_000.0
    }
}

#[async_trait]
impl BlockchainProvider for SolanaProvider {
    fn get_info(&self) -> BlockchainInfo {
        self.network_info.clone()
    }
    
    async fn initialize(&mut self, config: BlockchainConfig) -> Result<()> {
        // Initialize RPC client with provided endpoints
        let client = SolanaRpcClient::new(&config.rpc_endpoints[0])?;
        
        // Test connection
        client.get_version().await?;
        
        self.config = Some(config);
        self.rpc_client = Some(client);
        
        Ok(())
    }
    
    fn validate_address(&self, address: &str) -> Result<bool> {
        // Solana addresses are base58 encoded and typically 32-44 characters
        if address.len() < 32 || address.len() > 44 {
            return Ok(false);
        }
        
        // Basic base58 validation
        let valid_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        Ok(address.chars().all(|c| valid_chars.contains(c)))
    }
    
    async fn get_native_balance(&self, address: &str) -> Result<u64> {
        let client = self.rpc_client.as_ref()
            .ok_or_else(|| anyhow!("Solana provider not initialized"))?;
        
        let balance = client.get_balance(address).await?;
        Ok(balance)
    }
    
    async fn get_token_balance(&self, address: &str, token_contract: &str) -> Result<u64> {
        let client = self.rpc_client.as_ref()
            .ok_or_else(|| anyhow!("Solana provider not initialized"))?;
        
        // Get SPL token account balance
        let token_accounts = client.get_token_accounts_by_owner(address, token_contract).await?;
        
        if let Some(account) = token_accounts.first() {
            Ok(account.account.data.parsed.info.token_amount.amount.parse().unwrap_or(0))
        } else {
            Ok(0)
        }
    }
    
    async fn create_transaction(&self, request: TransactionRequest) -> Result<UnsignedTransaction> {
        let client = self.rpc_client.as_ref()
            .ok_or_else(|| anyhow!("Solana provider not initialized"))?;
        
        let (transaction, blockhash) = if let Some(token_contract) = &request.token_contract {
            // SPL token transfer
            client.create_spl_transfer_transaction(
                &request.from_address,
                &request.to_address,
                token_contract,
                request.amount,
            ).await?
        } else {
            // Native SOL transfer
            client.create_transfer_transaction(
                &request.from_address,
                &request.to_address,
                request.amount,
            ).await?
        };
        
        let estimated_fee = self.estimate_fees(&request).await?.total_fee;
        
        Ok(UnsignedTransaction {
            transaction_data: transaction,
            hash_to_sign: blockhash,
            estimated_fee,
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(2), // Solana blockhashes expire quickly
        })
    }
    
    async fn broadcast_transaction(&self, signed_tx: &str) -> Result<String> {
        let client = self.rpc_client.as_ref()
            .ok_or_else(|| anyhow!("Solana provider not initialized"))?;
        
        let signature = client.send_transaction(signed_tx).await?;
        Ok(signature)
    }
    
    async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus> {
        let client = self.rpc_client.as_ref()
            .ok_or_else(|| anyhow!("Solana provider not initialized"))?;
        
        let tx_status = client.get_signature_status(tx_hash).await?;
        
        let status = match tx_status.confirmation_status.as_deref() {
            Some("confirmed") | Some("finalized") => TxStatus::Confirmed,
            Some("processed") => TxStatus::Pending,
            _ => {
                if tx_status.err.is_some() {
                    TxStatus::Failed
                } else {
                    TxStatus::Pending
                }
            }
        };
          Ok(TransactionStatus {
            hash: tx_hash.to_string(),
            status: status.clone(),
            confirmations: if status == TxStatus::Confirmed { 1 } else { 0 },
            block_number: tx_status.slot,
            gas_used: None, // Solana doesn't use gas
            gas_price: None,
            timestamp: tx_status.block_time.map(|ts| {
                chrono::DateTime::from_timestamp(ts, 0).unwrap_or_default()
            }),
        })
    }
    
    async fn estimate_fees(&self, _request: &TransactionRequest) -> Result<FeeEstimate> {
        let client = self.rpc_client.as_ref()
            .ok_or_else(|| anyhow!("Solana provider not initialized"))?;
        
        // Get recent prioritization fees
        let recent_fees = client.get_recent_prioritization_fees().await?;
        let avg_fee = recent_fees.iter().map(|f| f.prioritization_fee).sum::<u64>() 
            / recent_fees.len().max(1) as u64;
        
        // Base fee for a simple transaction
        let base_fee = 5000; // 0.000005 SOL
        let priority_fee = avg_fee;
        let total_fee = base_fee + priority_fee;
        
        Ok(FeeEstimate {
            gas_limit: 200_000, // Solana compute units
            gas_price: priority_fee,
            total_fee,
            estimated_confirmation_time: 15, // ~15 seconds for finalization
        })
    }    async fn get_network_status(&self) -> Result<NetworkStatus> {
        let _client = self.rpc_client.as_ref()
            .ok_or_else(|| anyhow!("Solana provider not initialized"))?;
        
        // TODO: Implement proper network status retrieval
        // For now, return a placeholder implementation
        Ok(NetworkStatus {
            latest_block: 250000000, // Placeholder slot number
            network_congestion: CongestionLevel::Low,
            average_block_time: 1, // ~400ms for Solana
            recommended_gas_price: 5000,
        })
    }
}

/// Solana RPC client for real blockchain interactions
#[derive(Debug)]
pub struct SolanaRpcClient {
    rpc_url: String,
    client: reqwest::Client,
    commitment: String,
    connection_pool: Arc<DashMap<String, reqwest::Client>>,
}

impl SolanaRpcClient {
    pub fn new(rpc_url: &str) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
            
        Ok(Self {
            rpc_url: rpc_url.to_string(),
            client,
            commitment: "confirmed".to_string(),
            connection_pool: Arc::new(DashMap::new()),
        })
    }

    async fn send_rpc_request(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let response = self.client
            .post(&self.rpc_url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        
        if let Some(error) = result.get("error") {
            return Err(anyhow!("Solana RPC error: {}", error));
        }
        
        Ok(result["result"].clone())
    }

    pub async fn get_version(&self) -> Result<SolanaVersion> {
        let result = self.send_rpc_request("getVersion", serde_json::Value::Null).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn get_balance(&self, address: &str) -> Result<u64> {
        let params = serde_json::json!([
            address,
            {"commitment": self.commitment}
        ]);
        
        let result = self.send_rpc_request("getBalance", params).await?;
        Ok(result["value"].as_u64().unwrap_or(0))
    }

    pub async fn get_token_accounts_by_owner(&self, address: &str, token_mint: &str) -> Result<Vec<TokenAccount>> {
        let params = serde_json::json!([
            address,
            {"mint": token_mint},
            {"encoding": "jsonParsed", "commitment": self.commitment}
        ]);
        
        let result = self.send_rpc_request("getTokenAccountsByOwner", params).await?;
        let accounts: Vec<TokenAccount> = serde_json::from_value(result["value"].clone())?;
        Ok(accounts)
    }

    pub async fn create_transfer_transaction(&self, from: &str, to: &str, amount: u64) -> Result<(String, String)> {
        // Get recent blockhash
        let blockhash_result = self.send_rpc_request("getRecentBlockhash", serde_json::Value::Null).await?;
        let blockhash = blockhash_result["value"]["blockhash"].as_str()
            .ok_or_else(|| anyhow!("Failed to get blockhash"))?;

        // Create instruction for SOL transfer
        let instruction = serde_json::json!({
            "keys": [
                {"pubkey": from, "isSigner": true, "isWritable": true},
                {"pubkey": to, "isSigner": false, "isWritable": true}
            ],
            "programId": "11111111111111111111111111111112", // System Program
            "data": format!("{:08x}{:016x}", 2u32, amount) // Transfer instruction
        });

        let transaction = serde_json::json!({
            "feePayer": from,
            "recentBlockhash": blockhash,
            "instructions": [instruction]
        });

        Ok((serde_json::to_string(&transaction)?, blockhash.to_string()))
    }

    pub async fn create_spl_transfer_transaction(&self, from: &str, to: &str, token_mint: &str, amount: u64) -> Result<(String, String)> {
        // Get recent blockhash
        let blockhash_result = self.send_rpc_request("getRecentBlockhash", serde_json::Value::Null).await?;
        let blockhash = blockhash_result["value"]["blockhash"].as_str()
            .ok_or_else(|| anyhow!("Failed to get blockhash"))?;

        // SPL Token transfer instruction
        let instruction = serde_json::json!({
            "keys": [
                {"pubkey": from, "isSigner": false, "isWritable": true},
                {"pubkey": token_mint, "isSigner": false, "isWritable": false},
                {"pubkey": to, "isSigner": false, "isWritable": true},
                {"pubkey": from, "isSigner": true, "isWritable": false}
            ],
            "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", // SPL Token Program
            "data": format!("{:02x}{:016x}", 3u8, amount) // Transfer instruction
        });

        let transaction = serde_json::json!({
            "feePayer": from,
            "recentBlockhash": blockhash,
            "instructions": [instruction]
        });

        Ok((serde_json::to_string(&transaction)?, blockhash.to_string()))
    }

    pub async fn send_transaction(&self, signed_tx: &str) -> Result<String> {
        let params = serde_json::json!([
            signed_tx,
            {"encoding": "base64", "skipPreflight": false, "maxRetries": 5}
        ]);
        
        let result = self.send_rpc_request("sendTransaction", params).await?;
        Ok(result.as_str().unwrap_or("").to_string())
    }

    pub async fn get_signature_status(&self, signature: &str) -> Result<SignatureStatus> {
        let params = serde_json::json!([
            [signature],
            {"searchTransactionHistory": true}
        ]);
        
        let result = self.send_rpc_request("getSignatureStatuses", params).await?;
        let statuses: Vec<Option<SignatureStatus>> = serde_json::from_value(result["value"].clone())?;
        
        statuses.into_iter()
            .next()
            .flatten()
            .ok_or_else(|| anyhow!("Transaction not found"))
    }

    pub async fn get_recent_prioritization_fees(&self) -> Result<Vec<PrioritizationFee>> {
        let result = self.send_rpc_request("getRecentPrioritizationFees", serde_json::Value::Null).await?;
        Ok(serde_json::from_value(result)?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SolanaVersion {
    #[serde(rename = "solana-core")]
    pub solana_core: String,
    pub feature_set: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenAccount {
    pub account: TokenAccountData,
    pub pubkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenAccountData {
    pub data: TokenAccountInfo,
    pub executable: bool,
    pub lamports: u64,
    pub owner: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenAccountInfo {
    pub parsed: ParsedTokenAccount,
    pub program: String,
    pub space: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedTokenAccount {
    pub info: TokenAccountTokenInfo,
    #[serde(rename = "type")]
    pub account_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenAccountTokenInfo {
    #[serde(rename = "isNative")]
    pub is_native: bool,
    pub mint: String,
    pub owner: String,
    pub state: String,
    #[serde(rename = "tokenAmount")]
    pub token_amount: TokenAmount,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenAmount {
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: Option<f64>,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureStatus {
    pub slot: Option<u64>,
    pub confirmations: Option<u32>,
    pub err: Option<serde_json::Value>,
    #[serde(rename = "confirmationStatus")]
    pub confirmation_status: Option<String>,
    #[serde(rename = "blockTime")]
    pub block_time: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrioritizationFee {
    pub slot: u64,
    #[serde(rename = "prioritizationFee")]
    pub prioritization_fee: u64,
}

impl Default for SolanaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_address_validation() {
        let provider = SolanaProvider::new();
        
        // Valid Solana address
        assert!(provider.validate_address("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap());
        
        // Invalid addresses
        assert!(!provider.validate_address("invalid").unwrap());
        assert!(!provider.validate_address("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t").unwrap()); // Tron address
    }
    
    #[test]
    fn test_sol_conversion() {
        assert_eq!(SolanaProvider::sol_to_lamports(1.0), 1_000_000_000);
        assert_eq!(SolanaProvider::lamports_to_sol(1_000_000_000), 1.0);
    }
}
