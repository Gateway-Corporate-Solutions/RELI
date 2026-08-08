use sha2::{Digest, Sha256};

use crate::core::types::SignatureEnvelope;

pub fn sign_payload(payload: &[u8], signer_id: &str, key_id: &str) -> SignatureEnvelope {
    let mut hasher = Sha256::new();
    hasher.update(signer_id.as_bytes());
    hasher.update(b":");
    hasher.update(key_id.as_bytes());
    hasher.update(b":");
    hasher.update(payload);

    let digest = hasher.finalize();
    SignatureEnvelope {
        signer_id: signer_id.to_string(),
        signature_hex: format!("{:x}", digest),
        key_id: key_id.to_string(),
    }
}

pub fn verify_payload(signature: &SignatureEnvelope, payload: &[u8]) -> bool {
    let expected = sign_payload(payload, &signature.signer_id, &signature.key_id);
    expected.signature_hex == signature.signature_hex
}

#[cfg(test)]
mod tests {
    use super::{sign_payload, verify_payload};

    #[test]
    fn signature_verification_roundtrip_passes() {
        let payload = b"reli-payload";
        let sig = sign_payload(payload, "worker-001", "key-1");
        assert!(verify_payload(&sig, payload));
    }

    #[test]
    fn signature_verification_fails_for_tampered_payload() {
        let sig = sign_payload(b"payload-a", "worker-001", "key-1");
        assert!(!verify_payload(&sig, b"payload-b"));
    }
}
