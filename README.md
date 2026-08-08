# RELI: Reliability Token Network

RELI is a Rust reference implementation of a reliability marketplace for noisy data workflows.

The system uses off-chain workers for decoding and reconstruction, plus contract-facing lifecycle simulation for coordination, verification, dispute handling, and settlement. The goal is measurable reliability improvement with auditable economics.

## What RELI Solves

Many noisy-data pipelines are centralized and hard to audit end-to-end. RELI provides:

- Deterministic, schema-driven artifact flow.
- Multi-party verification and challenge handling.
- Quality-linked payout and slashing logic.
- Event-indexed lifecycle reconstruction for auditability.

## High-Level Architecture

1. Requester submits a job specification.
2. Workers execute algorithm profiles off-chain and submit signed outputs.
3. Verifiers validate submissions and evidence bundles.
4. Challengers can dispute low-quality or fraudulent outcomes.
5. Settlement computes quality-weighted rewards after lifecycle completion.

Heavy compute stays off-chain. Contract-facing logic tracks state transitions, disputes, and payouts.

## Current Implementation Status

The repository includes completed implementation through Phase 5:

- Phase 0: schema foundation, compatibility checks, deterministic fixtures, artifact validator.
- Phase 1: worker engine, algorithm registry, verifier and settlement integration, SDK alpha.
- Phase 2: lifecycle simulation (create/commit/reveal/verify/finalize/settle), event model, payout mirror checks.
- Phase 3: challenge workflow, slashing, latency SLO checks, collusion heuristics.
- Phase 4: pilot KPI assessment, telemetry schema, incident runbook, pilot reporting templates.
- Phase 5: multi-vertical profile registry, governance activation-delay and rollback simulation, scale benchmark suite, onboarding docs.

## Repository Map

- src/core: shared types, schema compatibility, attestation helpers, validation gate.
- src/algorithms: decode profile interface and default profile registry.
- src/worker: worker lifecycle and execution output models.
- src/verifier: submission, signature, evidence, and challenge validation logic.
- src/settlement: reliability scoring and reward share computation.
- src/contracts: contract-facing lifecycle simulator, event stream, dispute and slashing flow.
- src/sdk: requester and lifecycle client wrappers for integration flows.
- src/pilot: pilot KPI threshold assessment.
- src/registry: multi-vertical metric profile registry and compatibility checks.
- src/governance: proposal scheduling with activation delays and rollback drills.
- src/benchmarks: capacity benchmark evaluation and scaling-efficiency checks.
- docs: protocol spec, architecture policy, roadmap, schemas, fixtures, ADRs, and operations docs.

## Quick Start

Prerequisites:

- Rust toolchain (stable with Cargo)

Run full tests:

```bash
cargo test --locked
```

Run artifact and schema validation gate:

```bash
cargo run --locked --bin validate_phase0
```

Expected validator outcome: successful validation of all tracked artifacts and fixtures.

## Key Documentation

- docs/spec.md: protocol-level technical specification.
- docs/design.md: strict architecture policy and boundaries.
- docs/roadmap.md: phased execution status and exit criteria.
- docs/schemas/v1: canonical JSON schemas.
- docs/fixtures/v1: deterministic fixture corpus.
- docs/incident_runbook.md: pilot incident and rollback operations.
- docs/integration_guide.md: partner integration workflow.
- docs/onboarding_api_examples.md: onboarding-oriented Rust API sketches.

## Design Principles

- Keep compute-intensive decoding off-chain.
- Keep coordination, attestation, and settlement deterministic and auditable.
- Reward verified quality over raw compute volume.
- Enforce schema contracts and reproducible validation at each phase gate.

## Disclaimer

RELI is a protocol and systems engineering project. It is not financial advice. Production use in regulated or safety-critical domains requires legal, compliance, and domain-specific validation.
