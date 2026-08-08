use serde::{Deserialize, Serialize};

use crate::core::types::{HashRef, SignatureEnvelope};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSpec {
    pub job_id: String,
    pub requester: String,
    pub input_ref: HashRef,
    pub input_schema: String,
    pub algorithm_profile: String,
    pub metric_profile: String,
    pub min_worker_stake: u64,
    pub reward_pool: u64,
    pub commit_deadline_epoch_ms: u64,
    pub reveal_deadline_epoch_ms: u64,
    pub verify_deadline_epoch_ms: u64,
    pub finalize_deadline_epoch_ms: u64,
    pub privacy_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateSubmission {
    pub job_id: String,
    pub worker_id: String,
    pub output_ref: HashRef,
    pub output_summary_hash: HashRef,
    pub soft_evidence_ref: HashRef,
    pub algorithm_execution_hash: HashRef,
    pub worker_signature: SignatureEnvelope,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SoftEvidenceBundle {
    pub schema_version: String,
    pub algorithm_id: String,
    pub profile_id: String,
    pub confidence_score: f64,
    pub residual_score: f64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRecord {
    pub submission_id: String,
    pub verifier_id: String,
    pub checks_performed: Vec<String>,
    pub check_result: String,
    pub dispute_proof_ref: Option<HashRef>,
    pub verifier_signature: SignatureEnvelope,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChallengeRecord {
    pub challenge_id: String,
    pub submission_id: String,
    pub challenger_id: String,
    pub challenge_type: String,
    pub reason: String,
    pub observed_residual_score: Option<f64>,
    pub min_required_quality: Option<f64>,
    pub dispute_proof_ref: Option<HashRef>,
    pub replay_of_submission_id: Option<String>,
    pub challenger_signature: SignatureEnvelope,
}
