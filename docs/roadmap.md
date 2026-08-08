# RELI Roadmap (Execution Plan v1)

## 1. Purpose

This roadmap turns the RELI concept and technical specification into an execution plan with explicit deliverables, acceptance gates, metrics, and risk controls.

Scope covered:

- Protocol and product milestones.
- Rust implementation milestones.
- Smart-contract and settlement milestones.
- Pilot deployment and adoption milestones.

## 2. Planning Principles

- Reliability gain is the primary success metric.
- Heavy decoding remains off-chain across all phases.
- On-chain scope is coordination, attestation, settlement, and governance.
- Rewards are tied to verified quality, not raw compute volume.
- Every phase must end with measurable outcomes and a go/no-go gate.

## 3. Timeline Overview

The schedule below is written in relative months from project start.

1. Phase 0 (M0-M2): Foundations and reproducibility baseline.
2. Phase 1 (M2-M5): Core off-chain worker and schema stabilization.
3. Phase 2 (M5-M8): On-chain job lifecycle and staking.
4. Phase 3 (M8-M11): Verification, disputes, and slashing.
5. Phase 4 (M11-M14): Vertical pilot and production hardening.
6. Phase 5 (M14-M18): Scale-out, governance maturity, and broader launch.

## 4. Phase-by-Phase Plan

## 4.1 Phase 0: Foundations (M0-M2)

Objectives:

- Establish architecture baseline and repository conventions.
- Define canonical job, submission, and verification schemas.
- Lock deterministic algorithm test vector policy.

Deliverables:

- Initial protocol schema package with semantic versioning rules.
- Rust workspace skeleton under src with module boundaries from spec.
- Deterministic fixture set for at least Viterbi and one LDPC profile.
- Architecture Decision Record template and review workflow.

Exit Criteria:

- Reproducible test runs across at least 2 machine environments.
- Schema compatibility tests passing for upgrade and downgrade cases.
- Design review sign-off for off-chain/on-chain boundary.

Go/No-Go Gate:

- Go only if deterministic reproducibility and schema compatibility pass.

## 4.2 Phase 1: Worker Core (M2-M5)

Objectives:

- Build core worker engine for job intake, execution, and artifact packaging.
- Implement reliability-scoring primitives and confidence evidence handling.
- Deliver initial requester SDK for job submission and result fetch.

Deliverables:

- Worker service supporting JobSpec and CandidateSubmission flow.
- Algorithm plugin interface with versioned capability descriptors.
- Soft-evidence bundle format and verification helpers.
- SDK alpha with end-to-end local demo.

Exit Criteria:

- End-to-end local flow from noisy input to candidate output works.
- Worker can process at least 3 algorithm profiles via common interface.
- Artifact hashes and signatures verify correctly in integration tests.

Go/No-Go Gate:

- Go only if all critical integration tests are green and reproducible.

## 4.3 Phase 2: On-Chain Lifecycle (M5-M8)

Objectives:

- Implement contracts for job registration, commit/reveal, and settlement.
- Integrate staking constraints and basic reputation accounting.
- Add canonical aggregation and payout logic.

Deliverables:

- Smart-contract package for CreateJob, Commit, Reveal, Finalize, Settle.
- Contract-facing SDK bindings and simulation harness.
- Reference aggregation engine implementing reliability-weighted rules.
- On-chain event indexer for audit trail reconstruction.

Exit Criteria:

- Full job lifecycle succeeds in testnet simulation.
- Payout math exactly matches off-chain mirror tests.
- Event logs can rebuild complete job history deterministically.

Go/No-Go Gate:

- Go only if settlement invariants and accounting checks pass.

## 4.4 Phase 3: Verification and Adversarial Controls (M8-M11)

Objectives:

- Add verifier/challenger roles and dispute workflow.
- Introduce slashing for proven fraud and repeated low quality.
- Hardening against Sybil, collusion, and poisoning patterns.

Deliverables:

- Verification API and dispute-submission pipeline.
- Challenge window logic and fraud-proof artifact handling.
- Reputation progression and throttling policies.
- Adversarial simulation suite and attack playbooks.

Exit Criteria:

- Simulated fraud scenarios trigger expected slashing outcomes.
- False-positive dispute rate remains below defined threshold.
- Challenge processing latency remains within SLO.

Go/No-Go Gate:

- Go only if adversarial tests and slashing correctness pass.

## 4.5 Phase 4: Vertical Pilot (M11-M14)

Objectives:

- Run a narrow pilot in one high-fit industry vertical.
- Measure reliability gain, cost efficiency, and operational latency.
- Produce audit-ready evidence of provenance and settlement integrity.

Deliverables:

- Production pilot environment with monitored worker set.
- Domain-specific metric profile and oracle/validation policy.
- Pilot report including baseline comparisons and economic analysis.
- Incident response and rollback procedures validated.

Exit Criteria:

- Reliability improvement exceeds baseline target.
- Cost per useful reliability unit is competitive with centralized baseline.
- No unresolved critical security incidents.

Go/No-Go Gate:

- Go only if pilot demonstrates measurable economic and technical value.

## 4.6 Phase 5: Scale and Governance Maturity (M14-M18)

Objectives:

- Expand to multiple vertical profiles.
- Mature governance and upgrade procedures.
- Prepare for broader ecosystem integration.

Deliverables:

- Multi-profile algorithm and metric registry.
- Governance policy bundle with delayed activation and rollback controls.
- Public integration docs and partner onboarding toolkit.
- Capacity and performance benchmark report.

Exit Criteria:

- Multi-vertical support stable in production-like conditions.
- Governance upgrade drills completed successfully.
- Partner integration path validated by external teams.

Go/No-Go Gate:

- Go only if governance reliability and integration readiness are proven.

## 5. Workstreams and Owners

Core workstreams:

1. Protocol and contracts.
2. Worker runtime and algorithm modules.
3. Verification and adversarial defense.
4. Data schemas and artifact lifecycle.
5. SDKs and external integrations.
6. Observability, operations, and compliance.

Ownership model:

- Each workstream has one accountable owner.
- Cross-workstream dependencies must be tracked in weekly integration review.
- No milestone closes without owner sign-off and verification evidence.

## 6. Quality Gates and KPIs

Primary KPIs:

- Reliability gain over baseline (domain-specific).
- Residual-error reduction.
- End-to-end job latency.
- Verification cost as fraction of primary compute cost.
- Dispute accuracy (true fraud detection vs false positives).
- Uptime and successful-settlement rate.

Mandatory quality gates per release:

1. Deterministic reproducibility gate.
2. Security regression gate.
3. Contract invariant gate.
4. Backward-compatibility schema gate.
5. Pilot-metric gate for production promotions.

## 7. Security and Compliance Milestones

1. Threat model publication and quarterly refresh.
2. Independent contract and cryptography audits.
3. Data handling policy for sensitive payload tiers.
4. Incident response tabletop and live-fire drills.
5. Compliance evidence package suitable for regulated pilots.

## 8. Risks and Mitigations

Risk: Reliability scoring can be gamed by colluding workers.
Mitigation: Random verifier assignment, challenge incentives, correlation detection.

Risk: Verification cost becomes too high for frequent jobs.
Mitigation: Tiered verification, adaptive sampling, domain-specific fast checks.

Risk: Token demand remains speculative rather than utility-driven.
Mitigation: Require token for service access; focus on enterprise pilot demand.

Risk: Data privacy constraints block adoption in regulated sectors.
Mitigation: Encrypted artifacts, access controls, phased secure-compute integration.

Risk: Latency and throughput bottlenecks reduce usability.
Mitigation: Off-chain batching, queue prioritization, horizontal worker scaling.

## 9. Release Governance

- Monthly roadmap review with KPI delta and risk status.
- Phase gate decisions require explicit written sign-off.
- Breaking changes require migration plan and deprecation window.
- Emergency patches must include post-incident retrospective.

## 10. Definition of Success

RELI succeeds when it is cheaper or safer than centralized alternatives for at least one production domain, while preserving auditable provenance and high-confidence output quality under adversarial conditions.
