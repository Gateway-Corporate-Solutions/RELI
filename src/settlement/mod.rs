use std::collections::BTreeMap;

use crate::worker::WorkerExecutionReport;

#[derive(Debug, Clone, Copy)]
pub struct ScoreWeights {
    pub w1_accuracy: f64,
    pub w2_consensus: f64,
    pub w3_residual: f64,
    pub w4_uncertainty: f64,
    pub w5_penalty: f64,
}

pub fn reliability_score(
    weights: ScoreWeights,
    accuracy: f64,
    consensus: f64,
    residual: f64,
    uncertainty: f64,
    penalty: f64,
) -> f64 {
    (weights.w1_accuracy * accuracy)
        + (weights.w2_consensus * consensus)
        + (weights.w3_residual * residual)
        + (weights.w4_uncertainty * uncertainty)
        - (weights.w5_penalty * penalty)
}

pub fn payout_share(worker_score: f64, all_scores: &[f64]) -> f64 {
    let numerator = worker_score.max(0.0);
    let denominator: f64 = all_scores.iter().map(|score| score.max(0.0)).sum();

    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

#[derive(Debug, Clone, Copy)]
pub struct QualitySignal {
    pub accuracy: f64,
    pub consensus: f64,
    pub uncertainty: f64,
    pub penalty: f64,
}

#[derive(Debug, Clone)]
pub struct ScoredWorker {
    pub worker_id: String,
    pub score: f64,
    pub quality_score: f64,
}

pub fn score_execution_report(
    report: &WorkerExecutionReport,
    weights: ScoreWeights,
    signal: QualitySignal,
) -> ScoredWorker {
    let residual_improvement = report.quality_score;
    let score = reliability_score(
        weights,
        signal.accuracy,
        signal.consensus,
        residual_improvement,
        signal.uncertainty,
        signal.penalty,
    );

    ScoredWorker {
        worker_id: report.submission.worker_id.clone(),
        score,
        quality_score: report.quality_score,
    }
}

pub fn settle_reward_shares(scored_workers: &[ScoredWorker]) -> BTreeMap<String, f64> {
    let all_scores: Vec<f64> = scored_workers.iter().map(|worker| worker.score).collect();

    let mut payouts = BTreeMap::new();
    for worker in scored_workers {
        let share = payout_share(worker.score, &all_scores);
        payouts.insert(worker.worker_id.clone(), share);
    }

    payouts
}

#[cfg(test)]
mod tests {
    use crate::core::types::{HashRef, SignatureEnvelope};
    use crate::worker::models::CandidateSubmission;
    use crate::worker::WorkerExecutionReport;

    use super::{
        payout_share, reliability_score, score_execution_report, settle_reward_shares, QualitySignal,
        ScoreWeights,
    };

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn score_formula_matches_spec_shape() {
        let weights = ScoreWeights {
            w1_accuracy: 1.0,
            w2_consensus: 1.0,
            w3_residual: 1.0,
            w4_uncertainty: 1.0,
            w5_penalty: 1.0,
        };

        let score = reliability_score(weights, 0.9, 0.8, 0.7, 0.6, 0.5);
        assert!((score - 2.5).abs() < 1e-9);
    }

    #[test]
    fn payout_share_normalizes_positive_scores() {
        let share = payout_share(3.0, &[3.0, 1.0, -1.0]);
        assert!(approx_eq(share, 0.75));
    }

    #[test]
    fn score_is_monotonic_with_positive_accuracy_weight() {
        let weights = ScoreWeights {
            w1_accuracy: 0.8,
            w2_consensus: 0.4,
            w3_residual: 0.2,
            w4_uncertainty: 0.1,
            w5_penalty: 0.5,
        };

        let low_accuracy = reliability_score(weights, 0.2, 0.4, 0.3, 0.5, 0.1);
        let high_accuracy = reliability_score(weights, 0.7, 0.4, 0.3, 0.5, 0.1);

        assert!(high_accuracy > low_accuracy);
    }

    #[test]
    fn score_decreases_when_penalty_increases() {
        let weights = ScoreWeights {
            w1_accuracy: 1.0,
            w2_consensus: 1.0,
            w3_residual: 1.0,
            w4_uncertainty: 1.0,
            w5_penalty: 2.0,
        };

        let low_penalty = reliability_score(weights, 0.7, 0.7, 0.7, 0.7, 0.1);
        let high_penalty = reliability_score(weights, 0.7, 0.7, 0.7, 0.7, 0.5);

        assert!(high_penalty < low_penalty);
    }

    #[test]
    fn payout_shares_sum_to_one_when_positive_mass_exists() {
        let scores = [3.0, 2.0, 1.0, -4.0];
        let sum: f64 = scores.iter().map(|score| payout_share(*score, &scores)).sum();
        assert!(approx_eq(sum, 1.0));
    }

    #[test]
    fn payout_share_is_zero_for_non_positive_score() {
        let scores = [4.0, 2.0, 0.0, -1.0];
        assert!(approx_eq(payout_share(0.0, &scores), 0.0));
        assert!(approx_eq(payout_share(-1.0, &scores), 0.0));
    }

    #[test]
    fn payout_share_is_scale_invariant() {
        let base_scores = [2.0, 5.0, 1.0];
        let scaled_scores = [4.0, 10.0, 2.0];

        let base_share = payout_share(base_scores[1], &base_scores);
        let scaled_share = payout_share(scaled_scores[1], &scaled_scores);
        assert!(approx_eq(base_share, scaled_share));
    }

    fn sample_report(worker_id: &str, quality_score: f64) -> WorkerExecutionReport {
        WorkerExecutionReport {
            submission: CandidateSubmission {
                job_id: "job-001".to_string(),
                worker_id: worker_id.to_string(),
                output_ref: HashRef {
                    algorithm: "sha256".to_string(),
                    digest_hex: "aa".to_string(),
                },
                output_summary_hash: HashRef {
                    algorithm: "sha256".to_string(),
                    digest_hex: "bb".to_string(),
                },
                soft_evidence_ref: HashRef {
                    algorithm: "sha256".to_string(),
                    digest_hex: "cc".to_string(),
                },
                algorithm_execution_hash: HashRef {
                    algorithm: "sha256".to_string(),
                    digest_hex: "dd".to_string(),
                },
                worker_signature: SignatureEnvelope {
                    signer_id: worker_id.to_string(),
                    signature_hex: "sig".to_string(),
                    key_id: "key-1".to_string(),
                },
            },
            algorithm_id: "viterbi".to_string(),
            algorithm_profile: "viterbi-basic-bsc".to_string(),
            soft_evidence_bytes: b"{}".to_vec(),
            residual_score: 1.0 - quality_score,
            quality_score,
        }
    }

    #[test]
    fn scoring_from_execution_report_rewards_higher_quality() {
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

        let high = score_execution_report(&sample_report("worker-high", 0.9), weights, signal);
        let low = score_execution_report(&sample_report("worker-low", 0.2), weights, signal);

        assert!(high.score > low.score);
    }

    #[test]
    fn reward_shares_bias_toward_better_scored_workers() {
        let scored_workers = vec![
            super::ScoredWorker {
                worker_id: "worker-a".to_string(),
                score: 0.9,
                quality_score: 0.9,
            },
            super::ScoredWorker {
                worker_id: "worker-b".to_string(),
                score: 0.3,
                quality_score: 0.3,
            },
            super::ScoredWorker {
                worker_id: "worker-c".to_string(),
                score: -0.2,
                quality_score: 0.1,
            },
        ];

        let payouts = settle_reward_shares(&scored_workers);
        let a = payouts.get("worker-a").copied().unwrap_or_default();
        let b = payouts.get("worker-b").copied().unwrap_or_default();
        let c = payouts.get("worker-c").copied().unwrap_or_default();

        assert!(a > b);
        assert!(approx_eq(c, 0.0));
        assert!(approx_eq(a + b + c, 1.0));
    }
}
