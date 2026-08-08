# RELI Schemas

This directory contains versioned protocol schemas.

Layout:

- docs/schemas/v1/job_spec.schema.json
- docs/schemas/v1/candidate_submission.schema.json
- docs/schemas/v1/verification_record.schema.json
- docs/schemas/v1/algorithm_fixture.schema.json
- docs/schemas/v1/challenge_record.schema.json
- docs/schemas/v1/soft_evidence_bundle.schema.json
- docs/schemas/v1/contract_job_state.schema.json
- docs/schemas/v1/contract_event.schema.json
- docs/schemas/v1/pilot_telemetry.schema.json
- docs/schemas/v1/metric_profile.schema.json
- docs/schemas/v1/governance_proposal.schema.json
- docs/schemas/v1/capacity_benchmark_report.schema.json
- docs/schemas/v1/launch_readiness_report.schema.json
- docs/schemas/v1/tokenomics_policy.schema.json
- docs/schemas/v1/compliance_attestation.schema.json

Compatibility fixtures:

- docs/schemas/compat/job_spec.v1_compatible.schema.json
- docs/schemas/compat/job_spec.v1_breaking.schema.json

Versioning rules:

1. Patch version: backwards-compatible clarifications and optional fields.
2. Minor version: backwards-compatible additions.
3. Major version: breaking changes requiring migration.

Compatibility policy is defined in docs/design.md.
