use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::settlement::{score_execution_report, QualitySignal, ScoreWeights};
use crate::worker::models::JobSpec;
use crate::worker::WorkerExecutionReport;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractJobState {
    Created,
    CommitOpen,
    RevealOpen,
    VerifyOpen,
    Finalized,
    Settled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerAccount {
    pub worker_id: String,
    pub staked_amount: u64,
    pub reputation: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractEventKind {
    JobCreated,
    CommitOpened,
    SubmissionCommitted,
    RevealOpened,
    SubmissionRevealed,
    VerifyOpened,
    VerificationRecorded,
    ChallengeSubmitted,
    ChallengeResolved,
    StakeSlashed,
    JobFinalized,
    JobSettled,
    ReputationUpdated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    pub seq: u64,
    pub job_id: String,
    pub kind: ContractEventKind,
    pub actor_id: String,
    pub details: String,
}

#[derive(Debug, Clone)]
struct CommitRecord {
    commitment_hash: String,
    stake_locked: u64,
}

#[derive(Debug, Clone)]
pub struct FinalizationRecord {
    pub scored_workers: Vec<(String, f64)>,
    pub winner_worker_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeType {
    FabricatedOutput,
    LowQuality,
    ReplayAttack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeStatus {
    Open,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeRecord {
    pub challenge_id: String,
    pub job_id: String,
    pub challenger_id: String,
    pub target_worker_id: String,
    pub challenge_type: ChallengeType,
    pub evidence_score: f64,
    pub status: ChallengeStatus,
    pub submitted_seq: u64,
    pub resolved_seq: Option<u64>,
}

#[derive(Debug, Clone)]
struct ContractJob {
    spec: JobSpec,
    state: ContractJobState,
    commits: HashMap<String, CommitRecord>,
    reveals: HashMap<String, WorkerExecutionReport>,
    verification_pass_count: u32,
    open_challenge_ids: Vec<String>,
    finalization: Option<FinalizationRecord>,
    payouts: Option<BTreeMap<String, f64>>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContractError {
    #[error("job not found")]
    JobNotFound,
    #[error("worker not registered")]
    WorkerNotRegistered,
    #[error("insufficient stake for worker")]
    InsufficientStake,
    #[error("invalid job state transition")]
    InvalidState,
    #[error("commitment mismatch")]
    CommitmentMismatch,
    #[error("missing commit")]
    MissingCommit,
    #[error("no reveals available")]
    NoReveals,
    #[error("missing finalization")]
    MissingFinalization,
    #[error("challenge not found")]
    ChallengeNotFound,
    #[error("challenge state invalid")]
    InvalidChallengeState,
    #[error("target submission not found")]
    TargetSubmissionNotFound,
}

pub struct ContractLifecycle {
    jobs: HashMap<String, ContractJob>,
    workers: HashMap<String, WorkerAccount>,
    challenges: HashMap<String, ChallengeRecord>,
    verifier_worker_stats: HashMap<(String, String), (u64, u64)>,
    events: Vec<ContractEvent>,
    seq: u64,
    challenge_seq: u64,
}

impl ContractLifecycle {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            workers: HashMap::new(),
            challenges: HashMap::new(),
            verifier_worker_stats: HashMap::new(),
            events: Vec::new(),
            seq: 0,
            challenge_seq: 0,
        }
    }

    pub fn register_worker(&mut self, worker_id: &str, stake: u64) {
        self.workers.insert(
            worker_id.to_string(),
            WorkerAccount {
                worker_id: worker_id.to_string(),
                staked_amount: stake,
                reputation: 0,
            },
        );
    }

    fn push_event(&mut self, job_id: &str, kind: ContractEventKind, actor_id: &str, details: String) {
        self.seq += 1;
        self.events.push(ContractEvent {
            seq: self.seq,
            job_id: job_id.to_string(),
            kind,
            actor_id: actor_id.to_string(),
            details,
        });
    }

    pub fn create_job(&mut self, spec: JobSpec) -> String {
        let job_id = spec.job_id.clone();
        self.jobs.insert(
            job_id.clone(),
            ContractJob {
                spec,
                state: ContractJobState::Created,
                commits: HashMap::new(),
                reveals: HashMap::new(),
                verification_pass_count: 0,
                open_challenge_ids: Vec::new(),
                finalization: None,
                payouts: None,
            },
        );

        self.push_event(&job_id, ContractEventKind::JobCreated, "requester", "job created".to_string());
        self.open_commit_phase(&job_id).ok();
        job_id
    }

    pub fn open_commit_phase(&mut self, job_id: &str) -> Result<(), ContractError> {
        let job = self.jobs.get_mut(job_id).ok_or(ContractError::JobNotFound)?;
        if !matches!(job.state, ContractJobState::Created) {
            return Err(ContractError::InvalidState);
        }
        job.state = ContractJobState::CommitOpen;
        self.push_event(job_id, ContractEventKind::CommitOpened, "system", "commit phase open".to_string());
        Ok(())
    }

    pub fn commit_submission(
        &mut self,
        job_id: &str,
        worker_id: &str,
        commitment_hash: &str,
    ) -> Result<(), ContractError> {
        let worker = self.workers.get(worker_id).ok_or(ContractError::WorkerNotRegistered)?;
        let job = self.jobs.get_mut(job_id).ok_or(ContractError::JobNotFound)?;

        if !matches!(job.state, ContractJobState::CommitOpen) {
            return Err(ContractError::InvalidState);
        }
        if worker.staked_amount < job.spec.min_worker_stake {
            return Err(ContractError::InsufficientStake);
        }

        job.commits.insert(
            worker_id.to_string(),
            CommitRecord {
                commitment_hash: commitment_hash.to_string(),
                stake_locked: job.spec.min_worker_stake,
            },
        );

        self.push_event(
            job_id,
            ContractEventKind::SubmissionCommitted,
            worker_id,
            format!("commit={commitment_hash}"),
        );
        Ok(())
    }

    pub fn open_reveal_phase(&mut self, job_id: &str) -> Result<(), ContractError> {
        let job = self.jobs.get_mut(job_id).ok_or(ContractError::JobNotFound)?;
        if !matches!(job.state, ContractJobState::CommitOpen) {
            return Err(ContractError::InvalidState);
        }
        job.state = ContractJobState::RevealOpen;
        self.push_event(job_id, ContractEventKind::RevealOpened, "system", "reveal phase open".to_string());
        Ok(())
    }

    pub fn commitment_for_report(report: &WorkerExecutionReport) -> String {
        format!(
            "{}:{}:{}",
            report.submission.worker_id,
            report.submission.output_ref.digest_hex,
            report.submission.algorithm_execution_hash.digest_hex
        )
    }

    pub fn reveal_submission(
        &mut self,
        job_id: &str,
        report: WorkerExecutionReport,
    ) -> Result<(), ContractError> {
        let worker_id = report.submission.worker_id.clone();
        let job = self.jobs.get_mut(job_id).ok_or(ContractError::JobNotFound)?;

        if !matches!(job.state, ContractJobState::RevealOpen) {
            return Err(ContractError::InvalidState);
        }

        let commit = job.commits.get(&worker_id).ok_or(ContractError::MissingCommit)?;
        let expected = Self::commitment_for_report(&report);
        if commit.commitment_hash != expected {
            return Err(ContractError::CommitmentMismatch);
        }

        job.reveals.insert(worker_id.clone(), report);
        self.push_event(
            job_id,
            ContractEventKind::SubmissionRevealed,
            &worker_id,
            "reveal accepted".to_string(),
        );
        Ok(())
    }

    pub fn open_verify_phase(&mut self, job_id: &str) -> Result<(), ContractError> {
        let job = self.jobs.get_mut(job_id).ok_or(ContractError::JobNotFound)?;
        if !matches!(job.state, ContractJobState::RevealOpen) {
            return Err(ContractError::InvalidState);
        }
        job.state = ContractJobState::VerifyOpen;
        self.push_event(job_id, ContractEventKind::VerifyOpened, "system", "verify phase open".to_string());
        Ok(())
    }

    pub fn record_verification(
        &mut self,
        job_id: &str,
        verifier_id: &str,
        target_worker_id: &str,
        passed: bool,
    ) -> Result<(), ContractError> {
        let job = self.jobs.get_mut(job_id).ok_or(ContractError::JobNotFound)?;
        if !matches!(job.state, ContractJobState::VerifyOpen) {
            return Err(ContractError::InvalidState);
        }
        if !job.reveals.contains_key(target_worker_id) {
            return Err(ContractError::TargetSubmissionNotFound);
        }

        if passed {
            job.verification_pass_count += 1;
        }

        let stats = self
            .verifier_worker_stats
            .entry((verifier_id.to_string(), target_worker_id.to_string()))
            .or_insert((0, 0));
        if passed {
            stats.0 += 1;
        }
        stats.1 += 1;

        self.push_event(
            job_id,
            ContractEventKind::VerificationRecorded,
            verifier_id,
            if passed {
                format!("target={target_worker_id};pass")
            } else {
                format!("target={target_worker_id};fail")
            },
        );
        Ok(())
    }

    pub fn submit_challenge(
        &mut self,
        job_id: &str,
        challenger_id: &str,
        target_worker_id: &str,
        challenge_type: ChallengeType,
        evidence_score: f64,
    ) -> Result<String, ContractError> {
        let challenger = self
            .workers
            .get(challenger_id)
            .ok_or(ContractError::WorkerNotRegistered)?;
        let job = self.jobs.get_mut(job_id).ok_or(ContractError::JobNotFound)?;

        if !matches!(job.state, ContractJobState::VerifyOpen | ContractJobState::Finalized) {
            return Err(ContractError::InvalidState);
        }
        if !job.reveals.contains_key(target_worker_id) {
            return Err(ContractError::TargetSubmissionNotFound);
        }
        if challenger.staked_amount < (job.spec.min_worker_stake / 2).max(1) {
            return Err(ContractError::InsufficientStake);
        }

        self.challenge_seq += 1;
        let challenge_id = format!("challenge-{:04}", self.challenge_seq);
        let record = ChallengeRecord {
            challenge_id: challenge_id.clone(),
            job_id: job_id.to_string(),
            challenger_id: challenger_id.to_string(),
            target_worker_id: target_worker_id.to_string(),
            challenge_type,
            evidence_score,
            status: ChallengeStatus::Open,
            submitted_seq: self.seq + 1,
            resolved_seq: None,
        };

        job.open_challenge_ids.push(challenge_id.clone());
        self.challenges.insert(challenge_id.clone(), record);
        self.push_event(
            job_id,
            ContractEventKind::ChallengeSubmitted,
            challenger_id,
            format!("target={target_worker_id};evidence={evidence_score:.3}"),
        );
        Ok(challenge_id)
    }

    pub fn resolve_challenge(
        &mut self,
        challenge_id: &str,
        accepted: bool,
    ) -> Result<(), ContractError> {
        let challenge = self
            .challenges
            .get(challenge_id)
            .cloned()
            .ok_or(ContractError::ChallengeNotFound)?;

        if challenge.status != ChallengeStatus::Open {
            return Err(ContractError::InvalidChallengeState);
        }

        let job = self
            .jobs
            .get(&challenge.job_id)
            .ok_or(ContractError::JobNotFound)?;
        let slash_base = (job.spec.min_worker_stake / 5).max(1);
        let target_worker_id = challenge.target_worker_id.clone();
        let challenger_id = challenge.challenger_id.clone();
        let job_id = challenge.job_id.clone();

        let mut slash_events = Vec::<(String, u64, &'static str)>::new();
        if accepted {
            if let Some(target) = self.workers.get_mut(&target_worker_id) {
                let amount = slash_base.min(target.staked_amount);
                target.staked_amount = target.staked_amount.saturating_sub(amount);
                target.reputation -= 2;
                slash_events.push((target_worker_id.clone(), amount, "target"));
            }
            if let Some(challenger) = self.workers.get_mut(&challenger_id) {
                challenger.reputation += 1;
            }
        } else {
            if let Some(challenger) = self.workers.get_mut(&challenger_id) {
                let amount = slash_base.min(challenger.staked_amount);
                challenger.staked_amount = challenger.staked_amount.saturating_sub(amount);
                challenger.reputation -= 1;
                slash_events.push((challenger_id.clone(), amount, "challenger"));
            }
        }

        for (worker_id, amount, side) in slash_events {
            self.push_event(
                &job_id,
                ContractEventKind::StakeSlashed,
                &worker_id,
                format!("side={side};amount={amount}"),
            );
        }

        if let Some(challenge_mut) = self.challenges.get_mut(challenge_id) {
            challenge_mut.status = if accepted {
                ChallengeStatus::Accepted
            } else {
                ChallengeStatus::Rejected
            };
            challenge_mut.resolved_seq = Some(self.seq + 1);
        }

        if let Some(job_mut) = self.jobs.get_mut(&job_id) {
            job_mut.open_challenge_ids.retain(|id| id != challenge_id);
        }

        self.push_event(
            &job_id,
            ContractEventKind::ChallengeResolved,
            "system",
            format!("challenge_id={challenge_id};accepted={accepted}"),
        );

        Ok(())
    }

    pub fn challenge_false_positive_rate(&self) -> f64 {
        let mut rejected = 0_u64;
        let mut resolved = 0_u64;
        for challenge in self.challenges.values() {
            match challenge.status {
                ChallengeStatus::Rejected => {
                    rejected += 1;
                    resolved += 1;
                }
                ChallengeStatus::Accepted => {
                    resolved += 1;
                }
                ChallengeStatus::Open => {}
            }
        }

        if resolved == 0 {
            0.0
        } else {
            rejected as f64 / resolved as f64
        }
    }

    pub fn challenge_slo_ok(&self, max_event_delta: u64) -> bool {
        self.challenges.values().all(|challenge| {
            if let Some(resolved_seq) = challenge.resolved_seq {
                resolved_seq.saturating_sub(challenge.submitted_seq) <= max_event_delta
            } else {
                false
            }
        })
    }

    pub fn suspicious_verifier_worker_pairs(
        &self,
        min_interactions: u64,
        min_pass_ratio: f64,
    ) -> Vec<(String, String, f64, u64)> {
        let mut out = Vec::new();
        for ((verifier, worker), (passes, total)) in &self.verifier_worker_stats {
            if *total < min_interactions {
                continue;
            }
            let ratio = *passes as f64 / *total as f64;
            if ratio >= min_pass_ratio {
                out.push((verifier.clone(), worker.clone(), ratio, *total));
            }
        }
        out
    }

    pub fn finalize_job(
        &mut self,
        job_id: &str,
        weights: ScoreWeights,
        signals: &HashMap<String, QualitySignal>,
    ) -> Result<FinalizationRecord, ContractError> {
        let job = self.jobs.get_mut(job_id).ok_or(ContractError::JobNotFound)?;
        if !matches!(job.state, ContractJobState::VerifyOpen) {
            return Err(ContractError::InvalidState);
        }
        if job.reveals.is_empty() {
            return Err(ContractError::NoReveals);
        }

        let mut scored = Vec::<(String, f64)>::new();
        for report in job.reveals.values() {
            let signal = signals
                .get(&report.submission.worker_id)
                .copied()
                .unwrap_or(QualitySignal {
                    accuracy: 0.5,
                    consensus: 0.5,
                    uncertainty: 0.5,
                    penalty: 0.0,
                });
            let score = score_execution_report(report, weights, signal);
            scored.push((score.worker_id, score.score));
        }

        scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let winner = scored
            .first()
            .map(|entry| entry.0.clone())
            .ok_or(ContractError::NoReveals)?;

        let finalization = FinalizationRecord {
            scored_workers: scored,
            winner_worker_id: winner,
        };

        job.state = ContractJobState::Finalized;
        job.finalization = Some(finalization.clone());
        self.push_event(
            job_id,
            ContractEventKind::JobFinalized,
            "system",
            format!("winner={}", finalization.winner_worker_id),
        );

        Ok(finalization)
    }

    pub fn settle_job(&mut self, job_id: &str) -> Result<BTreeMap<String, f64>, ContractError> {
        let job = self.jobs.get_mut(job_id).ok_or(ContractError::JobNotFound)?;
        if !matches!(job.state, ContractJobState::Finalized) {
            return Err(ContractError::InvalidState);
        }
        if !job.open_challenge_ids.is_empty() {
            return Err(ContractError::InvalidChallengeState);
        }

        let scored = job
            .finalization
            .as_ref()
            .ok_or(ContractError::MissingFinalization)?
            .scored_workers
            .clone();

        let payouts = Self::contract_payout_shares(&scored);
        job.payouts = Some(payouts.clone());
        job.state = ContractJobState::Settled;

        let mut reputation_events = Vec::<(String, i64)>::new();
        for (worker_id, share) in &payouts {
            if let Some(account) = self.workers.get_mut(worker_id) {
                if *share > 0.0 {
                    account.reputation += 1;
                }
                reputation_events.push((worker_id.clone(), account.reputation));
            }
        }

        for (worker_id, reputation) in reputation_events {
            self.push_event(
                job_id,
                ContractEventKind::ReputationUpdated,
                &worker_id,
                format!("reputation={reputation}"),
            );
        }

        self.push_event(
            job_id,
            ContractEventKind::JobSettled,
            "system",
            format!("payout_recipients={}", payouts.len()),
        );

        Ok(payouts)
    }

    fn contract_payout_shares(scored_workers: &[(String, f64)]) -> BTreeMap<String, f64> {
        let positive_sum: f64 = scored_workers.iter().map(|(_, score)| score.max(0.0)).sum();
        let mut out = BTreeMap::new();

        for (worker_id, score) in scored_workers {
            let share = if positive_sum == 0.0 {
                0.0
            } else {
                score.max(0.0) / positive_sum
            };
            out.insert(worker_id.clone(), share);
        }

        out
    }

    pub fn get_job_state(&self, job_id: &str) -> Option<ContractJobState> {
        self.jobs.get(job_id).map(|job| job.state)
    }

    pub fn events(&self) -> &[ContractEvent] {
        &self.events
    }

    pub fn worker_reputation(&self, worker_id: &str) -> Option<i64> {
        self.workers.get(worker_id).map(|account| account.reputation)
    }

    pub fn locked_stake(&self, job_id: &str, worker_id: &str) -> Option<u64> {
        self.jobs
            .get(job_id)
            .and_then(|job| job.commits.get(worker_id))
            .map(|record| record.stake_locked)
    }
}

impl Default for ContractLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EventIndexer;

impl EventIndexer {
    pub fn timeline_for_job(events: &[ContractEvent], job_id: &str) -> Vec<ContractEvent> {
        events
            .iter()
            .filter(|event| event.job_id == job_id)
            .cloned()
            .collect()
    }

    pub fn rebuild_state(events: &[ContractEvent], job_id: &str) -> Option<ContractJobState> {
        let mut state = None;
        for event in events.iter().filter(|event| event.job_id == job_id) {
            state = match event.kind {
                ContractEventKind::JobCreated => Some(ContractJobState::Created),
                ContractEventKind::CommitOpened => Some(ContractJobState::CommitOpen),
                ContractEventKind::RevealOpened => Some(ContractJobState::RevealOpen),
                ContractEventKind::VerifyOpened => Some(ContractJobState::VerifyOpen),
                ContractEventKind::JobFinalized => Some(ContractJobState::Finalized),
                ContractEventKind::JobSettled => Some(ContractJobState::Settled),
                _ => state,
            };
        }
        state
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::core::types::HashRef;
    use crate::settlement::{settle_reward_shares, ScoredWorker};
    use crate::worker::models::JobSpec;
    use crate::worker::WorkerEngine;

    use super::{
        ChallengeType, ContractError, ContractJobState, ContractLifecycle, EventIndexer,
        QualitySignal, ScoreWeights,
    };

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
            min_worker_stake: 50,
            reward_pool: 1_000,
            commit_deadline_epoch_ms: 1,
            reveal_deadline_epoch_ms: 2,
            verify_deadline_epoch_ms: 3,
            finalize_deadline_epoch_ms: 4,
            privacy_mode: "public".to_string(),
        }
    }

    fn sample_weights() -> ScoreWeights {
        ScoreWeights {
            w1_accuracy: 0.4,
            w2_consensus: 0.2,
            w3_residual: 0.3,
            w4_uncertainty: 0.1,
            w5_penalty: 0.4,
        }
    }

    #[test]
    fn phase2_lifecycle_simulation_succeeds_end_to_end() {
        let mut lifecycle = ContractLifecycle::new();
        let engine = WorkerEngine::new();

        lifecycle.register_worker("worker-a", 100);
        lifecycle.register_worker("worker-b", 100);

        let job_id = lifecycle.create_job(sample_job("viterbi-basic-bsc"));
        assert_eq!(lifecycle.get_job_state(&job_id), Some(ContractJobState::CommitOpen));

        let report_a = engine
            .execute_job(&sample_job("viterbi-basic-bsc"), "worker-a", b"noisy-stream")
            .expect("report a");
        let report_b = engine
            .execute_job(&sample_job("turbo-basic"), "worker-b", b"noisy-stream")
            .expect("report b");

        let commit_a = ContractLifecycle::commitment_for_report(&report_a);
        let commit_b = ContractLifecycle::commitment_for_report(&report_b);

        lifecycle
            .commit_submission(&job_id, "worker-a", &commit_a)
            .expect("commit a");
        lifecycle
            .commit_submission(&job_id, "worker-b", &commit_b)
            .expect("commit b");

        assert_eq!(lifecycle.locked_stake(&job_id, "worker-a"), Some(50));

        lifecycle.open_reveal_phase(&job_id).expect("open reveal");
        lifecycle
            .reveal_submission(&job_id, report_a.clone())
            .expect("reveal a");
        lifecycle
            .reveal_submission(&job_id, report_b.clone())
            .expect("reveal b");

        lifecycle.open_verify_phase(&job_id).expect("open verify");
        lifecycle
            .record_verification(&job_id, "verifier-1", "worker-a", true)
            .expect("verify pass");
        lifecycle
            .record_verification(&job_id, "verifier-1", "worker-b", true)
            .expect("verify pass");

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

        let finalization = lifecycle
            .finalize_job(&job_id, sample_weights(), &signals)
            .expect("finalize");
        assert!(!finalization.winner_worker_id.is_empty());

        let payouts = lifecycle.settle_job(&job_id).expect("settle");
        let payout_sum: f64 = payouts.values().sum();
        assert!((payout_sum - 1.0).abs() < 1e-9);
        assert_eq!(lifecycle.get_job_state(&job_id), Some(ContractJobState::Settled));

        assert!(lifecycle.worker_reputation("worker-a").unwrap_or_default() >= 0);

        let rebuilt = EventIndexer::rebuild_state(lifecycle.events(), &job_id);
        assert_eq!(rebuilt, Some(ContractJobState::Settled));
    }

    #[test]
    fn payout_math_matches_offchain_mirror() {
        let scored = vec![
            ("worker-a".to_string(), 0.9_f64),
            ("worker-b".to_string(), 0.3_f64),
            ("worker-c".to_string(), -0.2_f64),
        ];

        let contract = super::ContractLifecycle::contract_payout_shares(&scored);

        let offchain = settle_reward_shares(&[
            ScoredWorker {
                worker_id: "worker-a".to_string(),
                score: 0.9,
                quality_score: 0.9,
            },
            ScoredWorker {
                worker_id: "worker-b".to_string(),
                score: 0.3,
                quality_score: 0.3,
            },
            ScoredWorker {
                worker_id: "worker-c".to_string(),
                score: -0.2,
                quality_score: 0.1,
            },
        ]);

        assert_eq!(contract.len(), offchain.len());
        for (worker, share) in &offchain {
            let contract_share = contract.get(worker).copied().unwrap_or_default();
            assert!((contract_share - share).abs() < 1e-9);
        }
    }

    #[test]
    fn event_timeline_rebuild_is_deterministic() {
        let mut lifecycle = ContractLifecycle::new();
        lifecycle.register_worker("worker-a", 100);
        let job_id = lifecycle.create_job(sample_job("viterbi-basic-bsc"));

        let state_before = EventIndexer::rebuild_state(lifecycle.events(), &job_id);
        assert_eq!(state_before, Some(ContractJobState::CommitOpen));

        let timeline = EventIndexer::timeline_for_job(lifecycle.events(), &job_id);
        let rebuilt_a = EventIndexer::rebuild_state(&timeline, &job_id);
        let rebuilt_b = EventIndexer::rebuild_state(&timeline, &job_id);
        assert_eq!(rebuilt_a, rebuilt_b);
    }

    #[test]
    fn insufficient_stake_blocks_commit() {
        let mut lifecycle = ContractLifecycle::new();
        lifecycle.register_worker("worker-low", 10);
        let job_id = lifecycle.create_job(sample_job("viterbi-basic-bsc"));

        let err = lifecycle
            .commit_submission(&job_id, "worker-low", "commit")
            .expect_err("should reject low stake");
        assert_eq!(err, ContractError::InsufficientStake);
    }

    #[test]
    fn accepted_challenge_slashes_target_and_blocks_settlement_until_resolution() {
        let mut lifecycle = ContractLifecycle::new();
        let engine = WorkerEngine::new();

        lifecycle.register_worker("worker-a", 120);
        lifecycle.register_worker("worker-b", 120);
        lifecycle.register_worker("challenger-1", 120);

        let job_id = lifecycle.create_job(sample_job("viterbi-basic-bsc"));
        let report_a = engine
            .execute_job(&sample_job("viterbi-basic-bsc"), "worker-a", b"noisy-stream")
            .expect("report a");
        let report_b = engine
            .execute_job(&sample_job("turbo-basic"), "worker-b", b"noisy-stream")
            .expect("report b");

        let commit_a = ContractLifecycle::commitment_for_report(&report_a);
        let commit_b = ContractLifecycle::commitment_for_report(&report_b);
        lifecycle
            .commit_submission(&job_id, "worker-a", &commit_a)
            .expect("commit a");
        lifecycle
            .commit_submission(&job_id, "worker-b", &commit_b)
            .expect("commit b");
        lifecycle.open_reveal_phase(&job_id).expect("open reveal");
        lifecycle
            .reveal_submission(&job_id, report_a)
            .expect("reveal a");
        lifecycle
            .reveal_submission(&job_id, report_b)
            .expect("reveal b");
        lifecycle.open_verify_phase(&job_id).expect("open verify");
        lifecycle
            .record_verification(&job_id, "verifier-1", "worker-a", true)
            .expect("verification");

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
                accuracy: 0.8,
                consensus: 0.8,
                uncertainty: 0.8,
                penalty: 0.0,
            },
        );

        lifecycle
            .finalize_job(&job_id, sample_weights(), &signals)
            .expect("finalize");

        let challenge_id = lifecycle
            .submit_challenge(
                &job_id,
                "challenger-1",
                "worker-a",
                ChallengeType::LowQuality,
                0.92,
            )
            .expect("challenge submit");

        let settle_err = lifecycle
            .settle_job(&job_id)
            .expect_err("settlement should be blocked by unresolved challenge");
        assert_eq!(settle_err, ContractError::InvalidChallengeState);

        let before = lifecycle.worker_reputation("worker-a").unwrap_or_default();
        lifecycle
            .resolve_challenge(&challenge_id, true)
            .expect("resolve accepted");
        let after = lifecycle.worker_reputation("worker-a").unwrap_or_default();
        assert!(after <= before - 1);

        let payouts = lifecycle.settle_job(&job_id).expect("settle after resolve");
        let payout_sum: f64 = payouts.values().sum();
        assert!((payout_sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn rejected_challenge_counts_toward_false_positive_rate() {
        let mut lifecycle = ContractLifecycle::new();
        let engine = WorkerEngine::new();

        lifecycle.register_worker("worker-a", 120);
        lifecycle.register_worker("challenger-1", 120);
        let job_id = lifecycle.create_job(sample_job("viterbi-basic-bsc"));
        let report_a = engine
            .execute_job(&sample_job("viterbi-basic-bsc"), "worker-a", b"noisy")
            .expect("report");
        let commit = ContractLifecycle::commitment_for_report(&report_a);
        lifecycle
            .commit_submission(&job_id, "worker-a", &commit)
            .expect("commit");
        lifecycle.open_reveal_phase(&job_id).expect("reveal open");
        lifecycle
            .reveal_submission(&job_id, report_a)
            .expect("reveal");
        lifecycle.open_verify_phase(&job_id).expect("verify open");

        let challenge_id = lifecycle
            .submit_challenge(
                &job_id,
                "challenger-1",
                "worker-a",
                ChallengeType::ReplayAttack,
                0.3,
            )
            .expect("challenge");
        lifecycle
            .resolve_challenge(&challenge_id, false)
            .expect("resolve rejected");

        let fp = lifecycle.challenge_false_positive_rate();
        assert!((fp - 1.0).abs() < 1e-9);
        assert!(lifecycle.challenge_slo_ok(3));
    }

    #[test]
    fn collusion_heuristic_flags_extreme_pass_pairing() {
        let mut lifecycle = ContractLifecycle::new();
        let engine = WorkerEngine::new();

        lifecycle.register_worker("worker-a", 200);
        lifecycle.register_worker("worker-b", 200);

        for i in 0..3 {
            let job_id = format!("job-collusion-{i}");
            let mut spec = sample_job("viterbi-basic-bsc");
            spec.job_id = job_id.clone();
            let created_id = lifecycle.create_job(spec);

            let report_a = engine
                .execute_job(&sample_job("viterbi-basic-bsc"), "worker-a", b"noisy")
                .expect("report a");
            let report_b = engine
                .execute_job(&sample_job("turbo-basic"), "worker-b", b"noisy")
                .expect("report b");
            let commit_a = ContractLifecycle::commitment_for_report(&report_a);
            let commit_b = ContractLifecycle::commitment_for_report(&report_b);
            lifecycle
                .commit_submission(&created_id, "worker-a", &commit_a)
                .expect("commit a");
            lifecycle
                .commit_submission(&created_id, "worker-b", &commit_b)
                .expect("commit b");
            lifecycle.open_reveal_phase(&created_id).expect("reveal open");
            lifecycle
                .reveal_submission(&created_id, report_a)
                .expect("reveal a");
            lifecycle
                .reveal_submission(&created_id, report_b)
                .expect("reveal b");
            lifecycle.open_verify_phase(&created_id).expect("verify open");
            lifecycle
                .record_verification(&created_id, "verifier-1", "worker-a", true)
                .expect("verify a pass");
            lifecycle
                .record_verification(&created_id, "verifier-1", "worker-b", false)
                .expect("verify b fail");
        }

        let suspicious = lifecycle.suspicious_verifier_worker_pairs(3, 0.95);
        assert!(suspicious
            .iter()
            .any(|(verifier, worker, _, _)| verifier == "verifier-1" && worker == "worker-a"));
    }
}
