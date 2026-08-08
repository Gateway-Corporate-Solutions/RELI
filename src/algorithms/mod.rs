use thiserror::Error;

use crate::worker::models::JobSpec;
use crate::worker::models::SoftEvidenceBundle;

pub trait DecoderAlgorithm {
    fn algorithm_id(&self) -> &'static str;
    fn supports(&self, profile_id: &str) -> bool;
    fn decode(&self, job: &JobSpec, input: &[u8]) -> DecodeResult;
}

#[derive(Debug, Clone)]
pub struct DecodeResult {
    pub algorithm_id: &'static str,
    pub profile_id: String,
    pub output_bytes: Vec<u8>,
    pub evidence_bytes: Vec<u8>,
    pub residual_score: f64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AlgorithmError {
    #[error("unsupported algorithm profile: {0}")]
    UnsupportedProfile(String),
}

pub struct AlgorithmRegistry {
    algorithms: Vec<Box<dyn DecoderAlgorithm + Send + Sync>>,
}

fn make_soft_evidence_bytes(
    algorithm_id: &str,
    profile_id: &str,
    confidence_score: f64,
    residual_score: f64,
    notes: Vec<String>,
) -> Vec<u8> {
    let bundle = SoftEvidenceBundle {
        schema_version: "1.0.0".to_string(),
        algorithm_id: algorithm_id.to_string(),
        profile_id: profile_id.to_string(),
        confidence_score,
        residual_score,
        notes,
    };

    serde_json::to_vec(&bundle).unwrap_or_else(|_| b"{}".to_vec())
}

impl AlgorithmRegistry {
    pub fn with_defaults() -> Self {
        Self {
            algorithms: vec![
                Box::new(ViterbiBasicBsc),
                Box::new(LdpcMinSumBasic),
                Box::new(TurboBasic),
            ],
        }
    }

    pub fn decode(
        &self,
        job: &JobSpec,
        input: &[u8],
    ) -> Result<DecodeResult, AlgorithmError> {
        let profile = job.algorithm_profile.as_str();
        for algorithm in &self.algorithms {
            if algorithm.supports(profile) {
                return Ok(algorithm.decode(job, input));
            }
        }

        Err(AlgorithmError::UnsupportedProfile(
            job.algorithm_profile.clone(),
        ))
    }

    pub fn supports_profile(&self, profile: &str) -> bool {
        self.algorithms.iter().any(|algorithm| algorithm.supports(profile))
    }
}

struct ViterbiBasicBsc;

impl DecoderAlgorithm for ViterbiBasicBsc {
    fn algorithm_id(&self) -> &'static str {
        "viterbi"
    }

    fn supports(&self, profile_id: &str) -> bool {
        profile_id == "viterbi-basic-bsc"
    }

    fn decode(&self, job: &JobSpec, input: &[u8]) -> DecodeResult {
        let output_bytes = input
            .iter()
            .enumerate()
            .map(|(idx, byte)| if idx % 5 == 0 { byte ^ 0x01 } else { *byte })
            .collect::<Vec<_>>();

        DecodeResult {
            algorithm_id: self.algorithm_id(),
            profile_id: job.algorithm_profile.clone(),
            output_bytes,
            evidence_bytes: make_soft_evidence_bytes(
                self.algorithm_id(),
                &job.algorithm_profile,
                0.875,
                0.125,
                vec!["path_metric=viterbi_stub".to_string()],
            ),
            residual_score: 0.125,
        }
    }
}

struct LdpcMinSumBasic;

impl DecoderAlgorithm for LdpcMinSumBasic {
    fn algorithm_id(&self) -> &'static str {
        "ldpc"
    }

    fn supports(&self, profile_id: &str) -> bool {
        profile_id == "ldpc-minsum-basic"
    }

    fn decode(&self, job: &JobSpec, input: &[u8]) -> DecodeResult {
        let output_bytes = input
            .iter()
            .enumerate()
            .map(|(idx, byte)| if idx % 7 == 0 { byte ^ 0x02 } else { *byte })
            .collect::<Vec<_>>();

        DecodeResult {
            algorithm_id: self.algorithm_id(),
            profile_id: job.algorithm_profile.clone(),
            output_bytes,
            evidence_bytes: make_soft_evidence_bytes(
                self.algorithm_id(),
                &job.algorithm_profile,
                0.9375,
                0.0625,
                vec![
                    "iterations=4".to_string(),
                    "decoder=minsum_stub".to_string(),
                ],
            ),
            residual_score: 0.0625,
        }
    }
}

struct TurboBasic;

impl DecoderAlgorithm for TurboBasic {
    fn algorithm_id(&self) -> &'static str {
        "turbo"
    }

    fn supports(&self, profile_id: &str) -> bool {
        profile_id == "turbo-basic"
    }

    fn decode(&self, job: &JobSpec, input: &[u8]) -> DecodeResult {
        let output_bytes = input
            .iter()
            .enumerate()
            .map(|(idx, byte)| if idx % 3 == 0 { byte ^ 0x04 } else { *byte })
            .collect::<Vec<_>>();

        DecodeResult {
            algorithm_id: self.algorithm_id(),
            profile_id: job.algorithm_profile.clone(),
            output_bytes,
            evidence_bytes: make_soft_evidence_bytes(
                self.algorithm_id(),
                &job.algorithm_profile,
                0.92,
                0.08,
                vec![
                    "iterations=6".to_string(),
                    "decoder=turbo_stub".to_string(),
                ],
            ),
            residual_score: 0.08,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::types::HashRef;
    use crate::worker::models::JobSpec;

    use super::{AlgorithmError, AlgorithmRegistry};

    fn sample_job(profile: &str) -> JobSpec {
        JobSpec {
            job_id: "job-001".to_string(),
            requester: "requester-001".to_string(),
            input_ref: HashRef {
                algorithm: "sha256".to_string(),
                digest_hex: "abcd".to_string(),
            },
            input_schema: "sensor.packet.v1".to_string(),
            algorithm_profile: profile.to_string(),
            metric_profile: "default-v1".to_string(),
            min_worker_stake: 1,
            reward_pool: 100,
            commit_deadline_epoch_ms: 1,
            reveal_deadline_epoch_ms: 2,
            verify_deadline_epoch_ms: 3,
            finalize_deadline_epoch_ms: 4,
            privacy_mode: "public".to_string(),
        }
    }

    #[test]
    fn default_registry_supports_three_profiles() {
        let registry = AlgorithmRegistry::with_defaults();
        assert!(registry.supports_profile("viterbi-basic-bsc"));
        assert!(registry.supports_profile("ldpc-minsum-basic"));
        assert!(registry.supports_profile("turbo-basic"));
    }

    #[test]
    fn decode_unsupported_profile_returns_error() {
        let registry = AlgorithmRegistry::with_defaults();
        let job = sample_job("unsupported-profile");
        let err = registry.decode(&job, b"input").expect_err("should fail");

        assert_eq!(
            err,
            AlgorithmError::UnsupportedProfile("unsupported-profile".to_string())
        );
    }
}
