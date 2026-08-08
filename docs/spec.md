# RELI Technical Specification (Draft v0.1)

## 1. Purpose and Scope

RELI defines a decentralized reliability marketplace for transforming noisy observations into higher-confidence outputs using classical error-correction and sequence-reconstruction algorithms.

This specification focuses on:

- Off-chain computation for heavy decoding.
- On-chain coordination, attestation, and settlement.
- Quality-weighted incentives and penalties.
- Reproducible, auditable reliability metrics.

This document does not prescribe a specific base blockchain or smart-contract language.

## 2. Terminology

- Noisy Input: Raw measurement stream or coded observations with uncertainty.
- Candidate Output: A worker-produced reconstructed/denoised result.
- Soft Evidence: Confidence information such as LLR vectors, residual statistics, or confidence intervals.
- Reliability Score: Scalar value representing quality contribution under defined metrics.
- Job: Unit of work posted by a requester.
- Worker: Node performing decoding/reconstruction.
- Verifier: Node validating outputs or challenging claims.
- Aggregator Rule: Deterministic function selecting canonical result.

## 3. System Goals

1. Convert noisy data into attested, higher-confidence data.
2. Align rewards with measurable reliability improvements.
3. Prevent profitable low-effort or fraudulent submissions.
4. Preserve provenance via immutable attestations.
5. Support domain-specific noise models and algorithm classes.

## 4. Non-Goals (Initial Phases)

- Running full turbo/LDPC/BCJR pipelines directly on-chain.
- Storing full raw data payloads on-chain.
- Solving every privacy regime in phase one.

## 5. Actors and Roles

- Requester:
	- Submits noisy-input job metadata and budget.
	- Defines acceptance policy and required confidence.
- Worker:
	- Stakes collateral.
	- Computes candidate output and evidence.
	- Submits signed result commitments.
- Verifier/Challenger:
	- Recomputes checks or spot-validates outputs.
	- Submits fraud proofs or dispute claims.
- Governance Participants:
	- Vote on algorithm registries, metric weights, and network parameters.

## 6. Architecture

### 6.1 Off-Chain Compute Plane

Workers execute algorithm modules such as:

- Viterbi decoding for most-likely state/sequence path.
- BCJR for posterior probabilities and soft output decoding.
- Turbo decoding for iterative code constraints.
- LDPC decoding (belief propagation / min-sum variants).
- Sparse recovery methods (matching pursuit, subspace methods) where applicable.

Compute outputs include:

- Reconstructed payload or sequence.
- Soft evidence and quality diagnostics.
- Deterministic execution metadata (algorithm id, parameter hash, version).

### 6.2 On-Chain Control Plane

Smart contracts handle:

- Job registration and escrow.
- Submission commitments and reveal windows.
- Aggregation finalization.
- Reward and slashing settlement.
- Reputation updates.

### 6.3 Storage Plane

- Large artifacts stored off-chain (for example, object storage or content-addressed storage).
- On-chain records contain content hashes and signed metadata.
- Optional coded redundancy for durable archival of cleaned outputs.

## 7. Job Lifecycle

1. CreateJob:
	 - Requester submits metadata, budget, stake constraints, deadline, metric profile.
2. CommitPhase:
	 - Workers submit commitments to candidate outputs.
3. RevealPhase:
	 - Workers reveal candidate outputs, evidence, and signatures.
4. VerifyPhase:
	 - Verifiers check evidence, run spot recomputation, or challenge suspicious results.
5. Finalize:
	 - Aggregator rule determines canonical output and scores.
6. Settle:
	 - Rewards distributed; penalties/slashing applied when needed.

## 8. Data Model (Logical)

### 8.1 JobSpec

- job_id: unique identifier.
- requester: account id.
- input_ref: hash or CID to noisy input.
- input_schema: format identifier and version.
- algorithm_profile: allowed algorithm set and parameter constraints.
- metric_profile: weights for quality scoring.
- min_worker_stake: minimum stake to submit.
- reward_pool: payment budget.
- deadlines: commit/reveal/verify/finalize timestamps.
- privacy_mode: public, restricted, or encrypted workflow.

### 8.2 CandidateSubmission

- job_id, worker_id.
- output_ref: hash or CID.
- output_summary_hash: compact summary hash for fast checks.
- soft_evidence_ref: confidence data reference.
- algorithm_execution_hash: algorithm id + params + binary/version digest.
- worker_signature.

### 8.3 VerificationRecord

- submission_id.
- verifier_id.
- checks_performed.
- check_result.
- dispute_proof_ref (optional).
- verifier_signature.

## 9. Reliability Scoring

Each submission receives a composite score:

$$
S = w_1 A + w_2 C + w_3 R + w_4 U - w_5 P
$$

Where:

- $A$: accuracy proxy (agreement with oracle or held-out validation).
- $C$: consensus consistency with independent high-quality submissions.
- $R$: residual noise reduction improvement.
- $U$: uncertainty calibration quality (confidence aligned with empirical error).
- $P$: penalties for anomalies, replay, or policy violations.

Weights $w_i$ are governance-configurable per domain profile.

## 10. Aggregation Rule

Default canonicalization can combine:

- Reliability-weighted voting for discrete outputs.
- Reliability-weighted averaging or smoothing for continuous streams.
- Bit-level LLR aggregation for coded payloads.

Example weighted vote for symbol $x$:

$$
\hat{x} = \arg\max_x \sum_i s_i \cdot \mathbf{1}(x_i = x)
$$

where $s_i$ is normalized submission reliability.

## 11. Rewards, Staking, and Slashing

### 11.1 Reward Distribution

Reward for worker $i$:

$$
R_i = B \cdot \frac{\max(S_i, 0)}{\sum_j \max(S_j, 0)}
$$

where $B$ is job reward budget after fees.

### 11.2 Staking Rules

- Minimum stake required for submission eligibility.
- Higher reputation may reduce required stake multipliers.
- Repeated high-quality performance unlocks larger job classes.

### 11.3 Slashing Conditions

- Proven fabricated output/evidence.
- Malicious collusion patterns.
- Failure to reveal after commit.
- Repeated low-quality submissions below threshold.

## 12. Verification and Trust Model

RELI supports phased verification strength:

- Tier 0: Multi-party agreement with statistical audits.
- Tier 1: Optimistic fraud proofs and challenge windows.
- Tier 2: Succinct proofs for deterministic subroutines where practical.

Initial pilots should use Tier 0 or Tier 1 before introducing expensive proving systems.

## 13. Security Considerations

- Sybil resistance via staking, identity costs, and reputation throttles.
- Data poisoning resistance via cross-worker diversity and anomaly detection.
- Collusion resistance via random verifier assignment and delayed reveal randomization.
- Adversarial-noise hardening through domain-specific robust metrics.
- Key management and signed artifact provenance for all submissions.

## 14. Privacy and Compliance

- Sensitive payloads remain off-chain.
- Support encrypted artifact references and access policies.
- Optionally integrate secure computation for regulated domains.
- Maintain auditable logs suitable for compliance review.

## 15. Governance

Governance controls:

- Algorithm registry additions/removals.
- Metric profile templates per industry vertical.
- Reward and slashing coefficient updates.
- Oracle provider admission and retirement.
- Upgrade paths for protocol and contract versions.

Governance changes should include staged activation delays and rollback procedures.

## 16. Performance Requirements (Pilot Targets)

- Job throughput: profile-dependent; target horizontal worker scalability.
- Finality latency: bounded by commit/reveal/verify windows.
- Verification cost: materially lower than full recomputation for common jobs.
- Availability: replicated artifact storage with integrity checks.

Pilot acceptance criteria should include measurable reliability gain, cost per job, and end-to-end latency.

## 17. Rust Implementation Guidance

Suggested module boundaries under src:

- core:
	- domain types, hashing, signatures, serialization.
- algorithms:
	- trait-based decoder interface and pluggable implementations.
- worker:
	- job intake, execution orchestration, evidence packaging.
- verifier:
	- independent checks, challenge proof construction.
- settlement:
	- score normalization, payout, slashing logic mirror tests.
- sdk:
	- requester APIs for submitting jobs and retrieving outputs.

Implementation requirements:

- Deterministic test vectors for every algorithm profile.
- Reproducible floating-point behavior policy (or fixed-point where needed).
- Structured tracing for auditability.
- Backward-compatible schema versioning.

## 18. API Sketch (Conceptual)

### 18.1 Requester API

- create_job(JobSpec) -> JobId
- get_job_status(JobId) -> JobState
- fetch_final_output(JobId) -> OutputRef + Attestation

### 18.2 Worker API

- claim_job(JobId)
- commit_submission(JobId, Commitment)
- reveal_submission(JobId, CandidateSubmission)

### 18.3 Verifier API

- fetch_pending_verifications() -> list
- submit_verification(VerificationRecord)
- submit_dispute(JobId, SubmissionId, ProofRef)

## 19. Industry Pilot Strategy

Recommended first pilots:

1. Industrial vibration or condition-monitoring streams.
2. Satellite packet reconstruction.

Pilot goals:

- Quantify residual error reduction against baseline.
- Demonstrate auditable provenance and dispute handling.
- Validate sustainable requester demand for reliability improvement.

## 20. Open Questions

- Which noise-model families should be standardized first?
- How should privacy tiers map to quality-verifiability tradeoffs?
- What minimum verifier quorum best balances cost and trust?
- Which proof systems are practical for selected algorithm subsets?
- How should reputation portability work across vertical domains?

## 21. Conclusion

RELI is best treated as a reliability infrastructure protocol, not a speculative asset wrapper. Its success depends on demonstrable reliability gains, robust off-chain engineering, credible verification economics, and strong integration with real industrial workflows.
