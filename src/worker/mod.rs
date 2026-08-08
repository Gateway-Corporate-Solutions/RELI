pub mod models;

use thiserror::Error;

use crate::algorithms::{AlgorithmError, AlgorithmRegistry};
use crate::core::attestation::sign_payload;
use crate::core::types::{HashRef, SignatureEnvelope};
use models::{CandidateSubmission, JobSpec};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobLifecycleState {
    Created,
    Committed,
    Revealed,
    Verified,
    Finalized,
    Settled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleError {
    JobIdMismatch,
    InvalidTransition {
        current: JobLifecycleState,
        attempted: JobLifecycleState,
    },
}

#[derive(Debug, Clone)]
pub struct WorkerJobSession {
    pub job_id: String,
    pub state: JobLifecycleState,
}

impl WorkerJobSession {
    pub fn new(job_id: impl Into<String>) -> Self {
        Self {
            job_id: job_id.into(),
            state: JobLifecycleState::Created,
        }
    }

    pub fn commit(&mut self, job_id: &str) -> Result<(), LifecycleError> {
        self.ensure_job_id(job_id)?;
        self.transition_to(JobLifecycleState::Committed)
    }

    pub fn reveal(&mut self, job_id: &str) -> Result<(), LifecycleError> {
        self.ensure_job_id(job_id)?;
        self.transition_to(JobLifecycleState::Revealed)
    }

    pub fn verify(&mut self, job_id: &str) -> Result<(), LifecycleError> {
        self.ensure_job_id(job_id)?;
        self.transition_to(JobLifecycleState::Verified)
    }

    pub fn finalize(&mut self, job_id: &str) -> Result<(), LifecycleError> {
        self.ensure_job_id(job_id)?;
        self.transition_to(JobLifecycleState::Finalized)
    }

    pub fn settle(&mut self, job_id: &str) -> Result<(), LifecycleError> {
        self.ensure_job_id(job_id)?;
        self.transition_to(JobLifecycleState::Settled)
    }

    fn ensure_job_id(&self, job_id: &str) -> Result<(), LifecycleError> {
        if self.job_id == job_id {
            Ok(())
        } else {
            Err(LifecycleError::JobIdMismatch)
        }
    }

    fn transition_to(&mut self, next: JobLifecycleState) -> Result<(), LifecycleError> {
        if self.state.can_transition_to(next) {
            self.state = next;
            Ok(())
        } else {
            Err(LifecycleError::InvalidTransition {
                current: self.state,
                attempted: next,
            })
        }
    }
}

impl JobLifecycleState {
    pub fn can_transition_to(self, next: JobLifecycleState) -> bool {
        matches!(
            (self, next),
            (JobLifecycleState::Created, JobLifecycleState::Committed)
                | (JobLifecycleState::Committed, JobLifecycleState::Revealed)
                | (JobLifecycleState::Revealed, JobLifecycleState::Verified)
                | (JobLifecycleState::Verified, JobLifecycleState::Finalized)
                | (JobLifecycleState::Finalized, JobLifecycleState::Settled)
        )
    }
}

#[derive(Debug, Clone)]
pub struct WorkerExecutionReport {
    pub submission: CandidateSubmission,
    pub algorithm_id: String,
    pub algorithm_profile: String,
    pub soft_evidence_bytes: Vec<u8>,
    pub residual_score: f64,
    pub quality_score: f64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WorkerExecutionError {
    #[error("algorithm decode failed: {0}")]
    Algorithm(#[from] AlgorithmError),
}

pub struct WorkerEngine {
    registry: AlgorithmRegistry,
}

impl WorkerEngine {
    pub fn new() -> Self {
        Self {
            registry: AlgorithmRegistry::with_defaults(),
        }
    }

    pub fn supports_profile(&self, profile: &str) -> bool {
        self.registry.supports_profile(profile)
    }

    pub fn commit_payload_hash(&self, payload: &[u8]) -> HashRef {
        use sha2::{Digest, Sha256};

        let digest = Sha256::digest(payload);
        HashRef {
            algorithm: "sha256".to_string(),
            digest_hex: format!("{:x}", digest),
        }
    }

    fn canonical_submission_payload(submission: &CandidateSubmission) -> Vec<u8> {
        format!(
            "{}|{}|{}|{}|{}|{}",
            submission.job_id,
            submission.worker_id,
            submission.output_ref.digest_hex,
            submission.output_summary_hash.digest_hex,
            submission.soft_evidence_ref.digest_hex,
            submission.algorithm_execution_hash.digest_hex
        )
        .into_bytes()
    }

    pub fn sign_submission_stub(&self, worker_id: &str) -> SignatureEnvelope {
        sign_payload(b"stub", worker_id, "worker-key-1")
    }

    pub fn build_submission_stub(&self, job: &JobSpec, worker_id: &str) -> CandidateSubmission {
        let signature = self.sign_submission_stub(worker_id);

        CandidateSubmission {
            job_id: job.job_id.clone(),
            worker_id: worker_id.to_string(),
            output_ref: self.commit_payload_hash(job.job_id.as_bytes()),
            output_summary_hash: self.commit_payload_hash(b"summary"),
            soft_evidence_ref: self.commit_payload_hash(b"soft-evidence"),
            algorithm_execution_hash: self.commit_payload_hash(b"algorithm-execution"),
            worker_signature: signature,
        }
    }

    pub fn execute_job(
        &self,
        job: &JobSpec,
        worker_id: &str,
        input: &[u8],
    ) -> Result<WorkerExecutionReport, WorkerExecutionError> {
        let decode = self.registry.decode(job, input)?;
        let mut submission = CandidateSubmission {
            job_id: job.job_id.clone(),
            worker_id: worker_id.to_string(),
            output_ref: self.commit_payload_hash(&decode.output_bytes),
            output_summary_hash: self.commit_payload_hash(
                format!("output_len={}", decode.output_bytes.len()).as_bytes(),
            ),
            soft_evidence_ref: self.commit_payload_hash(&decode.evidence_bytes),
            algorithm_execution_hash: self.commit_payload_hash(
                format!("{}:{}", decode.algorithm_id, decode.profile_id).as_bytes(),
            ),
            worker_signature: self.sign_submission_stub(worker_id),
        };
        let payload = Self::canonical_submission_payload(&submission);
        submission.worker_signature = sign_payload(&payload, worker_id, "worker-key-1");

        let residual_score = decode.residual_score;
        let quality_score = (1.0 - residual_score).clamp(0.0, 1.0);

        Ok(WorkerExecutionReport {
            submission,
            algorithm_id: decode.algorithm_id.to_string(),
            algorithm_profile: decode.profile_id,
            soft_evidence_bytes: decode.evidence_bytes,
            residual_score,
            quality_score,
        })
    }
}

impl Default for WorkerEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::types::HashRef;
    use crate::worker::models::JobSpec;

    use super::{JobLifecycleState, LifecycleError, WorkerJobSession};

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
            min_worker_stake: 10,
            reward_pool: 1000,
            commit_deadline_epoch_ms: 1,
            reveal_deadline_epoch_ms: 2,
            verify_deadline_epoch_ms: 3,
            finalize_deadline_epoch_ms: 4,
            privacy_mode: "public".to_string(),
        }
    }

    #[test]
    fn lifecycle_accepts_happy_path_transitions() {
        let mut session = WorkerJobSession::new("job-001");

        assert_eq!(session.state, JobLifecycleState::Created);
        assert!(session.commit("job-001").is_ok());
        assert!(session.reveal("job-001").is_ok());
        assert!(session.verify("job-001").is_ok());
        assert!(session.finalize("job-001").is_ok());
        assert!(session.settle("job-001").is_ok());
        assert_eq!(session.state, JobLifecycleState::Settled);
    }

    #[test]
    fn lifecycle_rejects_out_of_order_transition() {
        let mut session = WorkerJobSession::new("job-001");

        let err = session
            .reveal("job-001")
            .expect_err("reveal from created should fail");

        assert_eq!(
            err,
            LifecycleError::InvalidTransition {
                current: JobLifecycleState::Created,
                attempted: JobLifecycleState::Revealed,
            }
        );
    }

    #[test]
    fn lifecycle_rejects_job_id_mismatch() {
        let mut session = WorkerJobSession::new("job-001");

        let err = session
            .commit("job-002")
            .expect_err("commit with wrong job id should fail");
        assert_eq!(err, LifecycleError::JobIdMismatch);
    }

    #[test]
    fn terminal_state_rejects_further_transitions() {
        let mut session = WorkerJobSession::new("job-001");

        session.commit("job-001").unwrap();
        session.reveal("job-001").unwrap();
        session.verify("job-001").unwrap();
        session.finalize("job-001").unwrap();
        session.settle("job-001").unwrap();

        let err = session
            .settle("job-001")
            .expect_err("settle should not be repeatable");
        assert_eq!(
            err,
            LifecycleError::InvalidTransition {
                current: JobLifecycleState::Settled,
                attempted: JobLifecycleState::Settled,
            }
        );
    }

    #[test]
    fn worker_executes_multiple_profiles_via_common_interface() {
        let engine = super::WorkerEngine::new();
        let input = b"noisy-payload";

        for profile in ["viterbi-basic-bsc", "ldpc-minsum-basic", "turbo-basic"] {
            let job = sample_job(profile);
            let report = engine
                .execute_job(&job, "worker-001", input)
                .expect("profile should decode");

            assert_eq!(report.submission.job_id, job.job_id);
            assert_eq!(report.algorithm_profile, profile.to_string());
            assert!(report.quality_score >= 0.0 && report.quality_score <= 1.0);
            assert!(!report.soft_evidence_bytes.is_empty());
        }
    }
}
