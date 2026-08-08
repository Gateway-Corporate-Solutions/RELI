use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HashRef {
    pub algorithm: String,
    pub digest_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignatureEnvelope {
    pub signer_id: String,
    pub signature_hex: String,
    pub key_id: String,
}
