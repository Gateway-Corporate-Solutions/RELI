#[cfg(test)]
mod tests {
    use crate::core::types::HashRef;
    use crate::sdk::RequesterClient;
    use crate::settlement::{
        score_execution_report, settle_reward_shares, QualitySignal, ScoreWeights,
    };
    use crate::verifier::Verifier;
    use crate::worker::models::JobSpec;
    use crate::worker::{JobLifecycleState, WorkerEngine};

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
            min_worker_stake: 10,
            reward_pool: 1_000,
            commit_deadline_epoch_ms: 1,
            reveal_deadline_epoch_ms: 2,
            verify_deadline_epoch_ms: 3,
            finalize_deadline_epoch_ms: 4,
            privacy_mode: "public".to_string(),
        }
    }

    #[test]
    fn end_to_end_requester_worker_verifier_settlement_flow() {
        let mut client = RequesterClient::new();
        let engine = WorkerEngine::new();

        let job_id = client
            .create_job(sample_job("job-001", "viterbi-basic-bsc"))
            .expect("job creation should pass");

        client
            .transition_job_state(&job_id, JobLifecycleState::Committed)
            .expect("commit transition should pass");
        client
            .transition_job_state(&job_id, JobLifecycleState::Revealed)
            .expect("reveal transition should pass");

        let report_a = engine
            .execute_job(&sample_job("job-001", "viterbi-basic-bsc"), "worker-a", b"noisy-stream")
            .expect("worker-a execution should pass");
        let report_b = engine
            .execute_job(&sample_job("job-001", "turbo-basic"), "worker-b", b"noisy-stream")
            .expect("worker-b execution should pass");

        Verifier::validate_submission(&report_a.submission).expect("submission shape valid");
        Verifier::validate_submission_attestation(&report_a.submission)
            .expect("submission signature valid");
        Verifier::validate_soft_evidence_bundle(&report_a).expect("evidence valid");

        Verifier::validate_submission(&report_b.submission).expect("submission shape valid");
        Verifier::validate_submission_attestation(&report_b.submission)
            .expect("submission signature valid");
        Verifier::validate_soft_evidence_bundle(&report_b).expect("evidence valid");

        client.record_submission(report_a.submission.clone());
        client.record_submission(report_b.submission.clone());
        assert_eq!(client.list_submissions(&job_id).len(), 2);

        client
            .transition_job_state(&job_id, JobLifecycleState::Verified)
            .expect("verify transition should pass");

        let weights = ScoreWeights {
            w1_accuracy: 0.4,
            w2_consensus: 0.2,
            w3_residual: 0.3,
            w4_uncertainty: 0.1,
            w5_penalty: 0.4,
        };
        let signal = QualitySignal {
            accuracy: 0.8,
            consensus: 0.8,
            uncertainty: 0.8,
            penalty: 0.0,
        };

        let scored = vec![
            score_execution_report(&report_a, weights, signal),
            score_execution_report(&report_b, weights, signal),
        ];
        let payouts = settle_reward_shares(&scored);

        let payout_sum: f64 = payouts.values().sum();
        assert!((payout_sum - 1.0).abs() < 1e-9);

        client
            .transition_job_state(&job_id, JobLifecycleState::Finalized)
            .expect("finalize transition should pass");
        client
            .transition_job_state(&job_id, JobLifecycleState::Settled)
            .expect("settle transition should pass");

        assert_eq!(
            client.get_job_state(&job_id),
            Some(JobLifecycleState::Settled)
        );
    }
}
