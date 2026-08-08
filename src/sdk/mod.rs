use std::collections::HashMap;

use crate::contracts::{
    ChallengeType, ContractError, ContractLifecycle, ContractJobState,
};
use crate::settlement::{QualitySignal, ScoreWeights};
use crate::worker::models::{CandidateSubmission, JobSpec};
use crate::worker::{JobLifecycleState, WorkerEngine, WorkerExecutionError};

pub struct RequesterClient {
    jobs: HashMap<String, JobSpec>,
    states: HashMap<String, JobLifecycleState>,
    submissions: HashMap<String, Vec<CandidateSubmission>>,
}

impl RequesterClient {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            states: HashMap::new(),
            submissions: HashMap::new(),
        }
    }

    pub fn create_job(&mut self, job: JobSpec) -> Result<String, String> {
        if job.job_id.is_empty() {
            return Err("job_id is required".to_string());
        }

        let id = job.job_id.clone();
        self.states.insert(id.clone(), JobLifecycleState::Created);
        self.jobs.insert(id.clone(), job);
        Ok(id)
    }

    pub fn get_job_state(&self, job_id: &str) -> Option<JobLifecycleState> {
        self.states.get(job_id).copied()
    }

    pub fn transition_job_state(
        &mut self,
        job_id: &str,
        next: JobLifecycleState,
    ) -> Result<(), String> {
        let current = self
            .states
            .get(job_id)
            .copied()
            .ok_or_else(|| "unknown job_id".to_string())?;

        if current.can_transition_to(next) {
            self.states.insert(job_id.to_string(), next);
            Ok(())
        } else {
            Err(format!(
                "invalid state transition: {:?} -> {:?}",
                current, next
            ))
        }
    }

    pub fn record_submission(&mut self, submission: CandidateSubmission) {
        self.submissions
            .entry(submission.job_id.clone())
            .or_default()
            .push(submission);
    }

    pub fn list_submissions(&self, job_id: &str) -> &[CandidateSubmission] {
        self.submissions
            .get(job_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn run_worker_and_record(
        &mut self,
        engine: &WorkerEngine,
        job_id: &str,
        worker_id: &str,
        input: &[u8],
    ) -> Result<(), String> {
        let job = self
            .jobs
            .get(job_id)
            .ok_or_else(|| "unknown job_id".to_string())?;

        let report = engine
            .execute_job(job, worker_id, input)
            .map_err(|err: WorkerExecutionError| err.to_string())?;

        self.record_submission(report.submission);
        Ok(())
    }
}

impl Default for RequesterClient {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ContractLifecycleClient {
    lifecycle: ContractLifecycle,
}

impl ContractLifecycleClient {
    pub fn new() -> Self {
        Self {
            lifecycle: ContractLifecycle::new(),
        }
    }

    pub fn register_worker(&mut self, worker_id: &str, stake: u64) {
        self.lifecycle.register_worker(worker_id, stake);
    }

    pub fn create_job(&mut self, spec: JobSpec) -> String {
        self.lifecycle.create_job(spec)
    }

    pub fn commit_submission(
        &mut self,
        job_id: &str,
        worker_id: &str,
        commitment_hash: &str,
    ) -> Result<(), ContractError> {
        self.lifecycle
            .commit_submission(job_id, worker_id, commitment_hash)
    }

    pub fn open_reveal_phase(&mut self, job_id: &str) -> Result<(), ContractError> {
        self.lifecycle.open_reveal_phase(job_id)
    }

    pub fn reveal_submission(
        &mut self,
        job_id: &str,
        report: crate::worker::WorkerExecutionReport,
    ) -> Result<(), ContractError> {
        self.lifecycle.reveal_submission(job_id, report)
    }

    pub fn open_verify_phase(&mut self, job_id: &str) -> Result<(), ContractError> {
        self.lifecycle.open_verify_phase(job_id)
    }

    pub fn record_verification(
        &mut self,
        job_id: &str,
        verifier_id: &str,
        target_worker_id: &str,
        passed: bool,
    ) -> Result<(), ContractError> {
        self.lifecycle
            .record_verification(job_id, verifier_id, target_worker_id, passed)
    }

    pub fn submit_challenge(
        &mut self,
        job_id: &str,
        challenger_id: &str,
        target_worker_id: &str,
        challenge_type: ChallengeType,
        evidence_score: f64,
    ) -> Result<String, ContractError> {
        self.lifecycle.submit_challenge(
            job_id,
            challenger_id,
            target_worker_id,
            challenge_type,
            evidence_score,
        )
    }

    pub fn resolve_challenge(
        &mut self,
        challenge_id: &str,
        accepted: bool,
    ) -> Result<(), ContractError> {
        self.lifecycle.resolve_challenge(challenge_id, accepted)
    }

    pub fn finalize_job(
        &mut self,
        job_id: &str,
        weights: ScoreWeights,
        signals: &HashMap<String, QualitySignal>,
    ) -> Result<crate::contracts::FinalizationRecord, ContractError> {
        self.lifecycle.finalize_job(job_id, weights, signals)
    }

    pub fn settle_job(
        &mut self,
        job_id: &str,
    ) -> Result<std::collections::BTreeMap<String, f64>, ContractError> {
        self.lifecycle.settle_job(job_id)
    }

    pub fn get_job_state(&self, job_id: &str) -> Option<ContractJobState> {
        self.lifecycle.get_job_state(job_id)
    }

    pub fn inner(&self) -> &ContractLifecycle {
        &self.lifecycle
    }
}

impl Default for ContractLifecycleClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::contracts::ContractLifecycle;
    use crate::core::types::HashRef;
    use crate::settlement::{QualitySignal, ScoreWeights};
    use crate::worker::models::JobSpec;
    use crate::worker::{JobLifecycleState, WorkerEngine};

    use super::{ContractLifecycleClient, RequesterClient};

    fn sample_job(job_id: &str, profile: &str) -> JobSpec {
        JobSpec {
            job_id: job_id.to_string(),
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
    fn create_job_sets_initial_state() {
        let mut client = RequesterClient::new();
        let job_id = client
            .create_job(sample_job("job-001", "viterbi-basic-bsc"))
            .expect("create should succeed");

        assert_eq!(job_id, "job-001");
        assert_eq!(
            client.get_job_state(&job_id),
            Some(JobLifecycleState::Created)
        );
    }

    #[test]
    fn transition_state_enforces_order() {
        let mut client = RequesterClient::new();
        let job_id = client
            .create_job(sample_job("job-001", "viterbi-basic-bsc"))
            .expect("create should succeed");

        assert!(client
            .transition_job_state(&job_id, JobLifecycleState::Committed)
            .is_ok());
        assert!(client
            .transition_job_state(&job_id, JobLifecycleState::Finalized)
            .is_err());
    }

    #[test]
    fn worker_execution_is_recorded_in_sdk() {
        let mut client = RequesterClient::new();
        let engine = WorkerEngine::new();
        let job_id = client
            .create_job(sample_job("job-001", "turbo-basic"))
            .expect("create should succeed");

        client
            .run_worker_and_record(&engine, &job_id, "worker-001", b"noisy")
            .expect("worker should succeed");

        let submissions = client.list_submissions(&job_id);
        assert_eq!(submissions.len(), 1);
        assert_eq!(submissions[0].worker_id, "worker-001");
    }

    #[test]
    fn contract_lifecycle_sdk_binding_drives_phase2_flow() {
        let mut client = ContractLifecycleClient::new();
        let engine = WorkerEngine::new();

        client.register_worker("worker-a", 100);
        client.register_worker("worker-b", 100);
        let job_id = client.create_job(sample_job("job-001", "viterbi-basic-bsc"));

        let report_a = engine
            .execute_job(&sample_job("job-001", "viterbi-basic-bsc"), "worker-a", b"noisy")
            .expect("report a");
        let report_b = engine
            .execute_job(&sample_job("job-001", "turbo-basic"), "worker-b", b"noisy")
            .expect("report b");

        let commit_a = ContractLifecycle::commitment_for_report(&report_a);
        let commit_b = ContractLifecycle::commitment_for_report(&report_b);

        client
            .commit_submission(&job_id, "worker-a", &commit_a)
            .expect("commit a");
        client
            .commit_submission(&job_id, "worker-b", &commit_b)
            .expect("commit b");
        client.open_reveal_phase(&job_id).expect("open reveal");
        client
            .reveal_submission(&job_id, report_a)
            .expect("reveal a");
        client
            .reveal_submission(&job_id, report_b)
            .expect("reveal b");
        client.open_verify_phase(&job_id).expect("open verify");
        client
            .record_verification(&job_id, "verifier-1", "worker-a", true)
            .expect("verification");

        let weights = ScoreWeights {
            w1_accuracy: 0.4,
            w2_consensus: 0.2,
            w3_residual: 0.3,
            w4_uncertainty: 0.1,
            w5_penalty: 0.4,
        };
        let mut signals = HashMap::new();
        signals.insert(
            "worker-a".to_string(),
            QualitySignal {
                accuracy: 0.8,
                consensus: 0.8,
                uncertainty: 0.8,
                penalty: 0.0,
            },
        );
        signals.insert(
            "worker-b".to_string(),
            QualitySignal {
                accuracy: 0.7,
                consensus: 0.7,
                uncertainty: 0.7,
                penalty: 0.0,
            },
        );

        client
            .finalize_job(&job_id, weights, &signals)
            .expect("finalize");
        let payouts = client.settle_job(&job_id).expect("settle");

        assert_eq!(client.get_job_state(&job_id), Some(crate::contracts::ContractJobState::Settled));
        let payout_sum: f64 = payouts.values().sum();
        assert!((payout_sum - 1.0).abs() < 1e-9);
    }
}
