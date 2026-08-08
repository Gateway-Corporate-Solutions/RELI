use std::fs;
use std::path::{Path, PathBuf};

use jsonschema::JSONSchema;
use serde_json::Value;

use crate::core::schema_compat::assert_backward_compatible_schema;
use crate::core::types::{HashRef, SignatureEnvelope};
use crate::worker::models::{
    CandidateSubmission, ChallengeRecord, JobSpec, SoftEvidenceBundle, VerificationRecord,
};

#[derive(Debug)]
pub struct ValidationIssue {
    pub target: String,
    pub details: String,
}

impl ValidationIssue {
    fn new(target: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            details: details.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct ValidationSummary {
    pub checked: usize,
}

fn read_json(path: &Path) -> Result<Value, ValidationIssue> {
    let content = fs::read_to_string(path).map_err(|err| {
        ValidationIssue::new(path.display().to_string(), format!("read failed: {err}"))
    })?;

    serde_json::from_str::<Value>(&content).map_err(|err| {
        ValidationIssue::new(path.display().to_string(), format!("invalid json: {err}"))
    })
}

fn compile_schema(path: &Path) -> Result<JSONSchema, ValidationIssue> {
    let schema_value = read_json(path)?;
    JSONSchema::compile(&schema_value).map_err(|err| {
        ValidationIssue::new(path.display().to_string(), format!("invalid schema: {err}"))
    })
}

pub fn validate_file_against_schema(schema_path: &Path, target_path: &Path) -> Result<(), ValidationIssue> {
    let schema = compile_schema(schema_path)?;
    let payload = read_json(target_path)?;

    if let Err(errors) = schema.validate(&payload) {
        let joined = errors.map(|err| err.to_string()).collect::<Vec<_>>().join("; ");
        return Err(ValidationIssue::new(
            target_path.display().to_string(),
            format!("schema validation failed: {joined}"),
        ));
    }

    Ok(())
}

fn root_schema_path(root: &Path, file_name: &str) -> PathBuf {
    root.join("docs").join("schemas").join("v1").join(file_name)
}

fn build_sample_job_spec() -> JobSpec {
    JobSpec {
        job_id: "job-sample-0001".to_string(),
        requester: "requester-001".to_string(),
        input_ref: HashRef {
            algorithm: "sha256".to_string(),
            digest_hex: "abcd".to_string(),
        },
        input_schema: "sensor.packet.v1".to_string(),
        algorithm_profile: "viterbi-basic-bsc".to_string(),
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

fn build_sample_candidate_submission() -> CandidateSubmission {
    CandidateSubmission {
        job_id: "job-sample-0001".to_string(),
        worker_id: "worker-001".to_string(),
        output_ref: HashRef {
            algorithm: "sha256".to_string(),
            digest_hex: "00aa".to_string(),
        },
        output_summary_hash: HashRef {
            algorithm: "sha256".to_string(),
            digest_hex: "00bb".to_string(),
        },
        soft_evidence_ref: HashRef {
            algorithm: "sha256".to_string(),
            digest_hex: "00cc".to_string(),
        },
        algorithm_execution_hash: HashRef {
            algorithm: "sha256".to_string(),
            digest_hex: "00dd".to_string(),
        },
        worker_signature: SignatureEnvelope {
            signer_id: "worker-001".to_string(),
            signature_hex: "deadbeef".to_string(),
            key_id: "key-1".to_string(),
        },
    }
}

fn build_sample_verification_record() -> VerificationRecord {
    VerificationRecord {
        submission_id: "sub-0001".to_string(),
        verifier_id: "verifier-001".to_string(),
        checks_performed: vec!["shape_check".to_string()],
        check_result: "pass".to_string(),
        dispute_proof_ref: None,
        verifier_signature: SignatureEnvelope {
            signer_id: "verifier-001".to_string(),
            signature_hex: "faceb00c".to_string(),
            key_id: "key-2".to_string(),
        },
    }
}

fn build_sample_challenge_record() -> ChallengeRecord {
    ChallengeRecord {
        challenge_id: "challenge-001".to_string(),
        submission_id: "sub-0001".to_string(),
        challenger_id: "challenger-001".to_string(),
        challenge_type: "fabricated_output".to_string(),
        reason: "deterministic mismatch in evidence".to_string(),
        observed_residual_score: None,
        min_required_quality: None,
        dispute_proof_ref: Some(HashRef {
            algorithm: "sha256".to_string(),
            digest_hex: "proof-hash".to_string(),
        }),
        replay_of_submission_id: None,
        challenger_signature: SignatureEnvelope {
            signer_id: "challenger-001".to_string(),
            signature_hex: "feedface".to_string(),
            key_id: "key-3".to_string(),
        },
    }
}

fn build_sample_soft_evidence_bundle() -> SoftEvidenceBundle {
    SoftEvidenceBundle {
        schema_version: "1.0.0".to_string(),
        algorithm_id: "viterbi".to_string(),
        profile_id: "viterbi-basic-bsc".to_string(),
        confidence_score: 0.875,
        residual_score: 0.125,
        notes: vec!["path_metric=viterbi_stub".to_string()],
    }
}

fn build_sample_contract_job_state() -> Value {
    serde_json::json!({
        "job_id": "job-001",
        "state": "CommitOpen",
        "min_worker_stake": 50,
        "reward_pool": 1000,
        "verification_pass_count": 0
    })
}

fn build_sample_contract_event() -> Value {
    serde_json::json!({
        "seq": 1,
        "job_id": "job-001",
        "kind": "JobCreated",
        "actor_id": "requester-001",
        "details": "job created"
    })
}

fn build_sample_pilot_telemetry() -> Value {
    serde_json::json!({
        "job_id": "job-001",
        "timestamp_utc": "2026-08-07T12:00:00Z",
        "baseline_residual_error": 0.40,
        "reli_residual_error": 0.28,
        "baseline_cost_per_job": 1.00,
        "reli_cost_per_job": 0.95,
        "baseline_latency_ms": 800,
        "reli_latency_ms": 900,
        "critical_incident_count": 0,
        "challenges_submitted": 10,
        "challenges_accepted": 7,
        "challenges_rejected": 3
    })
}

fn build_sample_metric_profile() -> Value {
    serde_json::json!({
        "profile_id": "industrial-v1",
        "vertical": "industrial_iot",
        "schema_version": "1.0.0",
        "supported_algorithms": ["viterbi-basic-bsc", "ldpc-minsum-basic"],
        "max_latency_ms": 1000,
        "min_quality_score": 0.75
    })
}

fn build_sample_governance_proposal() -> Value {
    serde_json::json!({
        "proposal_id": "gov-0001",
        "title": "adjust quality floor",
        "status": "Scheduled",
        "submitted_epoch": 10,
        "activation_epoch": 12,
        "change": {
            "key": "quality_floor",
            "value": 0.8,
            "rollback_value": 0.7
        }
    })
}

fn build_sample_capacity_benchmark_report() -> Value {
    serde_json::json!({
        "profile_id": "industrial-v1",
        "samples": 3,
        "max_throughput_jps": 340.0,
        "p50_throughput_jps": 185.0,
        "scaling_efficiency": 0.85,
        "passes_scale_target": true
    })
}

fn build_sample_launch_readiness_report() -> Value {
    serde_json::json!({
        "ready": true,
        "score_percent": 100.0,
        "blockers": []
    })
}

fn build_sample_tokenomics_policy() -> Value {
    serde_json::json!({
        "version": "phase6-v1",
        "genesis_supply": 1_000_000_000,
        "staking_required": true,
        "slashing_enabled": true,
        "governance_activation_delay_epochs": 2
    })
}

fn build_sample_compliance_attestation() -> Value {
    serde_json::json!({
        "jurisdiction": "US",
        "legal_opinion_complete": true,
        "aml_kyc_policy_complete": true,
        "sanctions_screening_complete": true,
        "privacy_policy_complete": true,
        "reporting_controls_complete": true
    })
}

fn validate_sample_object(schema_path: &Path, value: Value, label: &str) -> Result<(), ValidationIssue> {
    let schema = compile_schema(schema_path)?;
    if let Err(errors) = schema.validate(&value) {
        let joined = errors.map(|err| err.to_string()).collect::<Vec<_>>().join("; ");
        return Err(ValidationIssue::new(label, joined));
    }

    Ok(())
}

pub fn validate_phase0_artifacts(root: &Path) -> Result<ValidationSummary, ValidationIssue> {
    let mut summary = ValidationSummary::default();

    let job_spec_schema = root_schema_path(root, "job_spec.schema.json");
    let candidate_schema = root_schema_path(root, "candidate_submission.schema.json");
    let verification_schema = root_schema_path(root, "verification_record.schema.json");
    let challenge_schema = root_schema_path(root, "challenge_record.schema.json");
    let soft_evidence_schema = root_schema_path(root, "soft_evidence_bundle.schema.json");
    let contract_state_schema = root_schema_path(root, "contract_job_state.schema.json");
    let contract_event_schema = root_schema_path(root, "contract_event.schema.json");
    let pilot_telemetry_schema = root_schema_path(root, "pilot_telemetry.schema.json");
    let metric_profile_schema = root_schema_path(root, "metric_profile.schema.json");
    let governance_proposal_schema = root_schema_path(root, "governance_proposal.schema.json");
    let capacity_benchmark_report_schema =
        root_schema_path(root, "capacity_benchmark_report.schema.json");
    let launch_readiness_report_schema =
        root_schema_path(root, "launch_readiness_report.schema.json");
    let tokenomics_policy_schema = root_schema_path(root, "tokenomics_policy.schema.json");
    let compliance_attestation_schema =
        root_schema_path(root, "compliance_attestation.schema.json");
    let fixture_schema = root_schema_path(root, "algorithm_fixture.schema.json");

    let compat_candidate = root
        .join("docs")
        .join("schemas")
        .join("compat")
        .join("job_spec.v1_compatible.schema.json");
    let breaking_candidate = root
        .join("docs")
        .join("schemas")
        .join("compat")
        .join("job_spec.v1_breaking.schema.json");

    validate_sample_object(
        &job_spec_schema,
        serde_json::to_value(build_sample_job_spec())
            .map_err(|err| ValidationIssue::new("sample job spec", err.to_string()))?,
        "sample job spec",
    )?;
    summary.checked += 1;

    validate_sample_object(
        &candidate_schema,
        serde_json::to_value(build_sample_candidate_submission())
            .map_err(|err| ValidationIssue::new("sample candidate", err.to_string()))?,
        "sample candidate",
    )?;
    summary.checked += 1;

    validate_sample_object(
        &verification_schema,
        serde_json::to_value(build_sample_verification_record())
            .map_err(|err| ValidationIssue::new("sample verification", err.to_string()))?,
        "sample verification",
    )?;
    summary.checked += 1;

    validate_sample_object(
        &challenge_schema,
        serde_json::to_value(build_sample_challenge_record())
            .map_err(|err| ValidationIssue::new("sample challenge", err.to_string()))?,
        "sample challenge",
    )?;
    summary.checked += 1;

    validate_sample_object(
        &soft_evidence_schema,
        serde_json::to_value(build_sample_soft_evidence_bundle())
            .map_err(|err| ValidationIssue::new("sample soft evidence", err.to_string()))?,
        "sample soft evidence",
    )?;
    summary.checked += 1;

    validate_sample_object(
        &contract_state_schema,
        build_sample_contract_job_state(),
        "sample contract state",
    )?;
    summary.checked += 1;

    validate_sample_object(
        &contract_event_schema,
        build_sample_contract_event(),
        "sample contract event",
    )?;
    summary.checked += 1;

    validate_sample_object(
        &pilot_telemetry_schema,
        build_sample_pilot_telemetry(),
        "sample pilot telemetry",
    )?;
    summary.checked += 1;

    validate_sample_object(
        &metric_profile_schema,
        build_sample_metric_profile(),
        "sample metric profile",
    )?;
    summary.checked += 1;

    validate_sample_object(
        &governance_proposal_schema,
        build_sample_governance_proposal(),
        "sample governance proposal",
    )?;
    summary.checked += 1;

    validate_sample_object(
        &capacity_benchmark_report_schema,
        build_sample_capacity_benchmark_report(),
        "sample capacity benchmark report",
    )?;
    summary.checked += 1;

    validate_sample_object(
        &launch_readiness_report_schema,
        build_sample_launch_readiness_report(),
        "sample launch readiness report",
    )?;
    summary.checked += 1;

    validate_sample_object(
        &tokenomics_policy_schema,
        build_sample_tokenomics_policy(),
        "sample tokenomics policy",
    )?;
    summary.checked += 1;

    validate_sample_object(
        &compliance_attestation_schema,
        build_sample_compliance_attestation(),
        "sample compliance attestation",
    )?;
    summary.checked += 1;

    assert_backward_compatible_schema(&job_spec_schema, &compat_candidate)?;
    summary.checked += 1;

    if assert_backward_compatible_schema(&job_spec_schema, &breaking_candidate).is_ok() {
        return Err(ValidationIssue::new(
            breaking_candidate.display().to_string(),
            "expected breaking candidate to fail compatibility check".to_string(),
        ));
    }
    summary.checked += 1;

    let fixtures_dir = root.join("docs").join("fixtures").join("v1");
    let entries = fs::read_dir(&fixtures_dir).map_err(|err| {
        ValidationIssue::new(fixtures_dir.display().to_string(), format!("read dir failed: {err}"))
    })?;

    for entry in entries {
        let entry = entry.map_err(|err| ValidationIssue::new("fixtures dir entry", err.to_string()))?;
        let path = entry.path();
        let is_fixture = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.ends_with(".fixture.json"))
            .unwrap_or(false);

        if !is_fixture {
            continue;
        }

        validate_file_against_schema(&fixture_schema, &path)?;
        summary.checked += 1;
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::validate_phase0_artifacts;

    #[test]
    fn phase0_artifacts_validate() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let summary = validate_phase0_artifacts(root).expect("phase0 validation should pass");
        assert!(summary.checked >= 18);
    }
}
