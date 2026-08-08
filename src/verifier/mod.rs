use thiserror::Error;

use crate::core::attestation::verify_payload;
use crate::worker::models::{CandidateSubmission, ChallengeRecord, VerificationRecord};
use crate::worker::models::SoftEvidenceBundle;
use crate::worker::WorkerExecutionReport;

pub struct Verifier;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum VerificationError {
    #[error("submission shape invalid")]
    InvalidSubmissionShape,
    #[error("verification record shape invalid")]
    InvalidVerificationRecord,
    #[error("challenge record invalid")]
    InvalidChallengeRecord,
    #[error("submission attestation invalid")]
    InvalidSubmissionAttestation,
    #[error("soft evidence bundle invalid")]
    InvalidSoftEvidenceBundle,
}

impl Verifier {
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

    pub fn verify_submission_shape(candidate: &CandidateSubmission) -> bool {
        !candidate.job_id.is_empty() && !candidate.worker_id.is_empty()
    }

    pub fn score_verification_result(record: &VerificationRecord) -> bool {
        !record.submission_id.is_empty() && !record.verifier_id.is_empty()
    }

    pub fn validate_submission(candidate: &CandidateSubmission) -> Result<(), VerificationError> {
        if Self::verify_submission_shape(candidate) {
            Ok(())
        } else {
            Err(VerificationError::InvalidSubmissionShape)
        }
    }

    pub fn validate_submission_attestation(
        candidate: &CandidateSubmission,
    ) -> Result<(), VerificationError> {
        let payload = Self::canonical_submission_payload(candidate);
        if verify_payload(&candidate.worker_signature, &payload) {
            Ok(())
        } else {
            Err(VerificationError::InvalidSubmissionAttestation)
        }
    }

    pub fn validate_soft_evidence_bundle(
        report: &WorkerExecutionReport,
    ) -> Result<SoftEvidenceBundle, VerificationError> {
        let bundle = serde_json::from_slice::<SoftEvidenceBundle>(&report.soft_evidence_bytes)
            .map_err(|_| VerificationError::InvalidSoftEvidenceBundle)?;

        let schema_ok = bundle.schema_version == "1.0.0";
        let algorithm_ok = bundle.algorithm_id == report.algorithm_id;
        let profile_ok = bundle.profile_id == report.algorithm_profile;
        let residual_ok = (bundle.residual_score - report.residual_score).abs() < 1e-9;
        let confidence_ok = (0.0..=1.0).contains(&bundle.confidence_score);

        if schema_ok && algorithm_ok && profile_ok && residual_ok && confidence_ok {
            Ok(bundle)
        } else {
            Err(VerificationError::InvalidSoftEvidenceBundle)
        }
    }

    pub fn validate_verification_record(
        record: &VerificationRecord,
    ) -> Result<(), VerificationError> {
        let valid_shape = Self::score_verification_result(record);
        let valid_result = matches!(record.check_result.as_str(), "pass" | "fail" | "inconclusive");
        let has_checks = !record.checks_performed.is_empty();

        if valid_shape && valid_result && has_checks {
            Ok(())
        } else {
            Err(VerificationError::InvalidVerificationRecord)
        }
    }

    pub fn validate_challenge_record(record: &ChallengeRecord) -> Result<(), VerificationError> {
        let has_identity = !record.challenge_id.is_empty()
            && !record.submission_id.is_empty()
            && !record.challenger_id.is_empty();
        let has_reason = record.reason.len() >= 8;
        let has_signature = !record.challenger_signature.signer_id.is_empty()
            && !record.challenger_signature.signature_hex.is_empty()
            && !record.challenger_signature.key_id.is_empty();

        if !(has_identity && has_reason && has_signature) {
            return Err(VerificationError::InvalidChallengeRecord);
        }

        match record.challenge_type.as_str() {
            "fabricated_output" => {
                if record.dispute_proof_ref.is_some() {
                    Ok(())
                } else {
                    Err(VerificationError::InvalidChallengeRecord)
                }
            }
            "low_quality" => {
                let residual = record.observed_residual_score;
                let min_quality = record.min_required_quality;
                match (residual, min_quality) {
                    (Some(r), Some(q)) => {
                        let residual_ok = (0.0..=1.0).contains(&r);
                        let quality_ok = (0.0..=1.0).contains(&q);
                        let fails_quality_floor = (1.0 - r) < q;
                        if residual_ok && quality_ok && fails_quality_floor {
                            Ok(())
                        } else {
                            Err(VerificationError::InvalidChallengeRecord)
                        }
                    }
                    _ => Err(VerificationError::InvalidChallengeRecord),
                }
            }
            "replay_attack" => {
                if let Some(prior) = record.replay_of_submission_id.as_deref() {
                    if !prior.is_empty() && prior != record.submission_id {
                        Ok(())
                    } else {
                        Err(VerificationError::InvalidChallengeRecord)
                    }
                } else {
                    Err(VerificationError::InvalidChallengeRecord)
                }
            }
            _ => Err(VerificationError::InvalidChallengeRecord),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::types::{HashRef, SignatureEnvelope};
    use crate::core::attestation::sign_payload;
    use crate::worker::models::{CandidateSubmission, ChallengeRecord, VerificationRecord};
    use crate::worker::{WorkerEngine, WorkerExecutionReport};
    use crate::core::types::HashRef as CoreHashRef;
    use crate::worker::models::JobSpec;

    use super::{VerificationError, Verifier};

    fn sample_submission() -> CandidateSubmission {
        let mut submission = CandidateSubmission {
            job_id: "job-001".to_string(),
            worker_id: "worker-001".to_string(),
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
                signer_id: "worker-001".to_string(),
                signature_hex: "dead".to_string(),
                key_id: "key-1".to_string(),
            },
        };

        let payload = format!(
            "{}|{}|{}|{}|{}|{}",
            submission.job_id,
            submission.worker_id,
            submission.output_ref.digest_hex,
            submission.output_summary_hash.digest_hex,
            submission.soft_evidence_ref.digest_hex,
            submission.algorithm_execution_hash.digest_hex
        );
        submission.worker_signature = sign_payload(payload.as_bytes(), "worker-001", "key-1");
        submission
    }

    fn sample_record() -> VerificationRecord {
        VerificationRecord {
            submission_id: "sub-001".to_string(),
            verifier_id: "verifier-001".to_string(),
            checks_performed: vec!["shape_check".to_string()],
            check_result: "pass".to_string(),
            dispute_proof_ref: None,
            verifier_signature: SignatureEnvelope {
                signer_id: "verifier-001".to_string(),
                signature_hex: "beef".to_string(),
                key_id: "key-2".to_string(),
            },
        }
    }

    fn sample_challenge(challenge_type: &str) -> ChallengeRecord {
        ChallengeRecord {
            challenge_id: "challenge-001".to_string(),
            submission_id: "sub-001".to_string(),
            challenger_id: "challenger-001".to_string(),
            challenge_type: challenge_type.to_string(),
            reason: "evidence indicates deterministic mismatch".to_string(),
            observed_residual_score: Some(0.9),
            min_required_quality: Some(0.4),
            dispute_proof_ref: Some(HashRef {
                algorithm: "sha256".to_string(),
                digest_hex: "proof-hash".to_string(),
            }),
            replay_of_submission_id: Some("sub-0004".to_string()),
            challenger_signature: SignatureEnvelope {
                signer_id: "challenger-001".to_string(),
                signature_hex: "feedface".to_string(),
                key_id: "key-3".to_string(),
            },
        }
    }

    #[test]
    fn verifier_accepts_valid_submission_and_record() {
        let submission = sample_submission();
        let record = sample_record();

        assert!(Verifier::validate_submission(&submission).is_ok());
        assert!(Verifier::validate_verification_record(&record).is_ok());
    }

    #[test]
    fn verifier_rejects_invalid_record_shape() {
        let mut record = sample_record();
        record.check_result = "unknown".to_string();

        let err = Verifier::validate_verification_record(&record).expect_err("should fail");
        assert_eq!(err, VerificationError::InvalidVerificationRecord);
    }

    #[test]
    fn verifier_rejects_invalid_submission_shape() {
        let mut submission = sample_submission();
        submission.job_id = String::new();

        let err = Verifier::validate_submission(&submission).expect_err("should fail");
        assert_eq!(err, VerificationError::InvalidSubmissionShape);
    }

    #[test]
    fn verifier_accepts_fabricated_output_challenge_with_proof() {
        let mut challenge = sample_challenge("fabricated_output");
        challenge.observed_residual_score = None;
        challenge.min_required_quality = None;
        challenge.replay_of_submission_id = None;

        assert!(Verifier::validate_challenge_record(&challenge).is_ok());
    }

    #[test]
    fn verifier_accepts_low_quality_challenge_when_threshold_is_violated() {
        let mut challenge = sample_challenge("low_quality");
        challenge.dispute_proof_ref = None;
        challenge.replay_of_submission_id = None;
        challenge.observed_residual_score = Some(0.8);
        challenge.min_required_quality = Some(0.5);

        assert!(Verifier::validate_challenge_record(&challenge).is_ok());
    }

    #[test]
    fn verifier_rejects_low_quality_challenge_without_quality_violation() {
        let mut challenge = sample_challenge("low_quality");
        challenge.dispute_proof_ref = None;
        challenge.replay_of_submission_id = None;
        challenge.observed_residual_score = Some(0.1);
        challenge.min_required_quality = Some(0.5);

        let err = Verifier::validate_challenge_record(&challenge).expect_err("should fail");
        assert_eq!(err, VerificationError::InvalidChallengeRecord);
    }

    #[test]
    fn verifier_accepts_replay_attack_challenge_with_prior_submission_ref() {
        let mut challenge = sample_challenge("replay_attack");
        challenge.dispute_proof_ref = None;
        challenge.observed_residual_score = None;
        challenge.min_required_quality = None;

        assert!(Verifier::validate_challenge_record(&challenge).is_ok());
    }

    #[test]
    fn verifier_rejects_unknown_challenge_type() {
        let challenge = sample_challenge("not-a-real-type");
        let err = Verifier::validate_challenge_record(&challenge).expect_err("should fail");
        assert_eq!(err, VerificationError::InvalidChallengeRecord);
    }

    #[test]
    fn verifier_validates_submission_attestation_and_detects_tamper() {
        let mut submission = sample_submission();
        assert!(Verifier::validate_submission_attestation(&submission).is_ok());

        submission.output_ref.digest_hex = "tampered".to_string();
        let err = Verifier::validate_submission_attestation(&submission).expect_err("should fail");
        assert_eq!(err, VerificationError::InvalidSubmissionAttestation);
    }

    fn sample_job(profile: &str) -> JobSpec {
        JobSpec {
            job_id: "job-001".to_string(),
            requester: "requester-001".to_string(),
            input_ref: CoreHashRef {
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
    fn verifier_validates_soft_evidence_bundle() {
        let engine = WorkerEngine::new();
        let job = sample_job("turbo-basic");
        let report = engine
            .execute_job(&job, "worker-001", b"noisy")
            .expect("worker should run");

        let bundle = Verifier::validate_soft_evidence_bundle(&report).expect("bundle should validate");
        assert_eq!(bundle.profile_id, "turbo-basic");
    }

    #[test]
    fn verifier_rejects_malformed_soft_evidence_bundle() {
        let bad_report = WorkerExecutionReport {
            submission: sample_submission(),
            algorithm_id: "turbo".to_string(),
            algorithm_profile: "turbo-basic".to_string(),
            soft_evidence_bytes: b"not-json".to_vec(),
            residual_score: 0.08,
            quality_score: 0.92,
        };

        let err =
            Verifier::validate_soft_evidence_bundle(&bad_report).expect_err("should fail");
        assert_eq!(err, VerificationError::InvalidSoftEvidenceBundle);
    }
}
