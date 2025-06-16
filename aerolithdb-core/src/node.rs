use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::config::NodeConfig;

/// Represents a node in the aerolithsDB network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub identity: NodeIdentity,
    pub status: NodeStatus,
    pub metadata: NodeMetadata,
    pub capabilities: NodeCapabilities,
}

/// Unique identifier for a node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

/// Node identity with cryptographic keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIdentity {
    pub signing_keypair: SigningKeyPair,
    pub box_keypair: BoxKeyPair,
    pub identity_proof: String,
}

/// Current status of the node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Starting,
    Active,
    Degraded,
    Leaving,
    Stopped,
}

/// Node metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub name: String,
    pub version: String,
    pub started_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub address: SocketAddr,
    pub external_address: Option<SocketAddr>,
}

/// Node capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub storage: bool,
    pub compute: bool,
    pub gateway: bool,
    pub bootstrap: bool,
    pub max_connections: usize,
    pub supported_protocols: Vec<String>,
}

/// Signing key pair for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningKeyPair {
    pub private_key: String, // base58 encoded
    pub public_key: String,  // base58 encoded
}

/// Box key pair for encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxKeyPair {
    pub private_key: String, // base58 encoded
    pub public_key: String,  // base58 encoded
}

impl Node {
    /// Create a new node with the given configuration
    pub async fn new(config: &NodeConfig) -> Result<Self> {
        let id = NodeId(Uuid::parse_str(&config.node_id)?);
        
        // Generate or load cryptographic identity
        let identity = NodeIdentity::generate().await?;
        
        let status = NodeStatus::Starting;
        
        let address = format!("{}:{}", config.bind_address, config.port).parse()?;
        let external_address = config.external_address
            .as_ref()
            .map(|addr| addr.parse())
            .transpose()?;
        
        let metadata = NodeMetadata {
            name: format!("aerolithsdb-node-{}", id.0),
            version: env!("CARGO_PKG_VERSION").to_string(),
            started_at: Utc::now(),
            last_seen: Utc::now(),
            address,
            external_address,
        };

        let capabilities = NodeCapabilities {
            storage: true,
            compute: true,
            gateway: true,
            bootstrap: false,
            max_connections: 100,
            supported_protocols: vec![
                "aerolithsdb/1.0.0".to_string(),
                "kad/1.0.0".to_string(),
                "gossipsub/1.1.0".to_string(),
            ],
        };

        Ok(Self {
            id,
            identity,
            status,
            metadata,
            capabilities,
        })
    }

    /// Update the node's last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.metadata.last_seen = Utc::now();
    }

    /// Set the node status
    pub fn set_status(&mut self, status: NodeStatus) {
        self.status = status;
        self.update_last_seen();
    }

    /// Get the node's public key for signing
    pub fn signing_public_key(&self) -> &str {
        &self.identity.signing_keypair.public_key
    }

    /// Get the node's public key for encryption
    pub fn box_public_key(&self) -> &str {
        &self.identity.box_keypair.public_key
    }

    /// Check if the node is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, NodeStatus::Active)
    }

    /// Check if the node can accept new connections
    pub fn can_accept_connections(&self) -> bool {
        matches!(self.status, NodeStatus::Active | NodeStatus::Degraded)
    }
}

impl NodeIdentity {    /// Generate a new cryptographic identity
    pub async fn generate() -> Result<Self> {        
        use dryoc::dryocbox;
        use dryoc::sign::{SigningKeyPair as DryocSigningKeyPair, PublicKey, SecretKey};
        use dryoc::types::Bytes;
        use base58::ToBase58;
        
        // Generate signing keypair
        let signing_keypair: DryocSigningKeyPair<PublicKey, SecretKey> = DryocSigningKeyPair::gen();
        let signing_public = signing_keypair.public_key.as_slice().to_base58();
        let signing_private = signing_keypair.secret_key.as_slice().to_base58();// Generate box keypair
        let box_keypair = dryocbox::KeyPair::gen();
        let box_public = box_keypair.public_key.as_slice().to_base58();
        let box_private = box_keypair.secret_key.as_slice().to_base58();

        // Create identity proof (simplified for now)
        let identity_proof = format!("proof_{}", uuid::Uuid::new_v4());

        Ok(Self {
            signing_keypair: SigningKeyPair {
                private_key: signing_private,
                public_key: signing_public,
            },
            box_keypair: BoxKeyPair {
                private_key: box_private,
                public_key: box_public,
            },
            identity_proof,
        })
    }

    /// Load identity from wallet file
    pub async fn load_from_wallet(wallet_path: &str) -> Result<Self> {
        let content = tokio::fs::read_to_string(wallet_path).await?;
        let wallet: serde_json::Value = serde_json::from_str(&content)?;
        
        // Extract identity from wallet format
        let identity = wallet["identity"].as_object()
            .ok_or_else(|| anyhow::anyhow!("Invalid wallet format"))?;

        let signing_keypair = SigningKeyPair {
            private_key: identity["signing_keypair"]["private"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing signing private key"))?
                .to_string(),
            public_key: identity["signing_keypair"]["public"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing signing public key"))?
                .to_string(),
        };

        let box_keypair = BoxKeyPair {
            private_key: identity["box_keypair"]["private"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing box private key"))?
                .to_string(),
            public_key: identity["box_keypair"]["public"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing box public key"))?
                .to_string(),
        };

        let identity_proof = identity["identity_proof"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing identity proof"))?
            .to_string();

        Ok(Self {
            signing_keypair,
            box_keypair,
            identity_proof,
        })
    }

    /// Save identity to wallet file
    pub async fn save_to_wallet(&self, wallet_path: &str) -> Result<()> {
        let wallet = serde_json::json!({
            "identity": {
                "signing_keypair": {
                    "private": self.signing_keypair.private_key,
                    "public": self.signing_keypair.public_key
                },
                "box_keypair": {
                    "private": self.box_keypair.private_key,
                    "public": self.box_keypair.public_key
                },
                "identity_proof": self.identity_proof
            },
            "capabilities": [],
            "key_rotation": {
                "current_version": 1,
                "next_rotation": chrono::Utc::now() + chrono::Duration::days(30)
            }
        });

        let content = serde_json::to_string_pretty(&wallet)?;
        tokio::fs::write(wallet_path, content).await?;
        Ok(())
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for NodeId {
    fn from(uuid: Uuid) -> Self {
        NodeId(uuid)
    }
}

impl From<NodeId> for Uuid {
    fn from(node_id: NodeId) -> Self {
        node_id.0
    }
}
