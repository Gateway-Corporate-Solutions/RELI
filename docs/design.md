# RELI Architecture Design Policy (Strict)

## 1. Policy Status

This document is normative. The keywords MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are interpreted as mandatory policy language.

Any change that violates a MUST or MUST NOT requirement is non-compliant and cannot be merged unless a formally approved exception is recorded.

## 2. Policy Objectives

- Preserve integrity of reliability outcomes.
- Keep protocol behavior deterministic and auditable.
- Minimize trust assumptions in worker outputs.
- Ensure secure and economically sound settlement.
- Prevent architectural drift from core design principles.

## 3. Core Architectural Invariants

1. Heavy decoding and reconstruction MUST execute off-chain.
2. On-chain logic MUST be limited to coordination, attestation, settlement, and governance.
3. Raw large payloads MUST NOT be stored directly on-chain.
4. Every accepted result MUST be traceable to immutable input and output hashes.
5. Reward distribution MUST be quality-weighted and reproducible.
6. Verification cost SHOULD be materially lower than full primary recomputation.

## 4. System Boundary Policy

### 4.1 Off-Chain Compute Boundary

- Worker modules MUST expose deterministic inputs, outputs, and execution metadata.
- Algorithm implementations MUST declare version, parameter profile, and reproducibility constraints.
- Worker outputs MUST include confidence evidence compatible with scoring policy.
- Non-deterministic behavior MUST be explicitly controlled, documented, and tested.

### 4.2 On-Chain Boundary

- Contracts MUST validate signatures, commitments, lifecycle transitions, and settlement invariants.
- Contracts MUST NOT perform computationally heavy iterative decoding.
- Contract state transitions MUST be finite, explicit, and covered by invariant tests.
- Contract upgradeability MUST include delay, notification, and rollback procedure.

### 4.3 Storage Boundary

- Artifacts MUST be content-addressed or hash-addressed.
- Input and output references MUST include integrity digests.
- Sensitive payloads SHOULD be encrypted off-chain with auditable access policy.
- Data retention and deletion policies MUST be documented per deployment profile.

## 5. Data and Schema Policy

- All protocol schemas MUST be versioned.
- Backward compatibility MUST be maintained within a declared major version.
- Breaking schema changes MUST include migration tooling and rollback strategy.
- Unknown fields SHOULD be preserved when forwarding messages unless prohibited by security policy.
- All externally visible IDs MUST be globally unique within their namespace.

## 6. Determinism and Reproducibility Policy

- Critical scoring and settlement paths MUST be deterministic.
- Floating-point behavior MUST be constrained by documented reproducibility rules or replaced with fixed-point where required.
- Reference test vectors MUST exist for every supported algorithm profile.
- Reproducibility tests MUST pass across at least two distinct execution environments before release.

## 7. Security Policy

### 7.1 Identity, Auth, and Signing

- All submissions, verifications, and disputes MUST be signed.
- Key rotation policy MUST be defined and tested.
- Replay resistance MUST be enforced for signed payloads.

### 7.2 Adversarial Resilience

- The system MUST include controls against Sybil attacks, collusion, and poisoning.
- Verifier selection SHOULD avoid deterministic predictability where it increases attack surface.
- Challenge windows MUST be long enough for practical dispute submission under expected network conditions.

### 7.3 Secret and Privacy Handling

- Secrets MUST NOT be logged in plaintext.
- Sensitive fields MUST be redacted in diagnostic outputs.
- Privacy controls MUST be profile-driven and enforceable per job class.

## 8. Incentive and Economic Policy

- Reward equations MUST be transparent and reproducible from public evidence.
- Slashing conditions MUST be explicit, testable, and non-ambiguous.
- No reward mechanism MAY rely solely on raw compute effort without quality evidence.
- Reputation changes MUST be explainable and derived from documented events.

## 9. Verification and Dispute Policy

- Each finalized job MUST have a defined verification path.
- Dispute submission MUST include proof references and traceable metadata.
- Fraud-proof evaluation MUST be deterministic.
- Unresolved disputes MUST block final settlement for the affected submission scope.
- Verifier and challenger incentives MUST discourage spam and griefing.

## 10. Reliability Metric Policy

- Reliability score components MUST be explicitly defined per metric profile.
- Metric weights MUST be governance-controlled and versioned.
- Calibration quality SHOULD be measured where confidence values are used.
- Domain profiles MUST define acceptable residual-noise thresholds.
- Metric changes MUST include regression analysis against baseline datasets.

## 11. Performance and Capacity Policy

- Service-level objectives MUST be defined for latency, throughput, and availability.
- Capacity planning MUST include worst-case verify-window load.
- Backpressure mechanisms MUST exist for worker and verifier queues.
- Production promotions MUST include benchmark evidence from realistic workloads.

## 12. Observability and Auditability Policy

- Critical events MUST emit structured logs with stable field names.
- Traceability from input hash to final settlement MUST be reconstructable.
- All policy-relevant decisions MUST produce machine-auditable evidence.
- Time synchronization assumptions MUST be documented for event ordering.

## 13. Governance and Change Control Policy

- Any change to scoring, slashing, or settlement logic MUST pass formal review.
- Governance proposals MUST include security impact and migration impact statements.
- Emergency changes MUST include expiration policy and post-incident review.
- Deprecated interfaces MUST have published removal timelines.

## 14. Rust Engineering Policy

- Unsafe Rust MUST be avoided unless justified and reviewed.
- Every unsafe block MUST include rationale and targeted tests.
- Public interfaces MUST be documented and versioned.
- Error handling MUST avoid silent failure in protocol-critical paths.
- Concurrency primitives MUST be selected to avoid race-prone shared mutability.

## 15. Testing Policy

Required test categories:

1. Unit tests for domain logic and invariants.
2. Integration tests for full job lifecycle.
3. Property tests for scoring and settlement algebra.
4. Fuzz tests for parser and schema robustness.
5. Adversarial scenario tests for fraud, collusion, and poisoning.

Release blocker conditions:

- Any failing invariant test in settlement or lifecycle logic.
- Any reproducibility regression for supported profiles.
- Any unresolved critical severity security issue.

## 16. Compliance and Regulatory Policy

- Deployment profiles MUST declare jurisdiction and data-handling assumptions.
- Healthcare, mobility, and other safety-critical profiles MUST include domain-specific validation gates.
- Liability boundaries and service guarantees MUST be clearly documented for pilot partners.

## 17. Exception Policy

- Exceptions to MUST or MUST NOT requirements require written Architecture Exception Record.
- Each exception MUST include reason, risk assessment, compensating controls, and expiration date.
- Expired exceptions MUST be resolved or renewed before subsequent release.

## 18. Architecture Review Checklist

Every architecture-affecting change MUST answer yes to all required checks:

1. Is heavy compute still off-chain?
2. Are on-chain changes minimal and settlement-safe?
3. Are hashes, signatures, and provenance complete?
4. Is scoring deterministic and reproducible?
5. Are dispute and slashing semantics unambiguous?
6. Is backward compatibility preserved or explicitly migrated?
7. Are security and adversarial impacts assessed?
8. Are observability and audit artifacts sufficient?

## 19. Enforcement Model

- CI MUST enforce tests, linting, schema checks, and reproducibility checks.
- Pull requests touching protocol-critical code MUST receive architecture review approval.
- Non-compliant changes MUST be blocked from merge unless an active exception exists.

## 20. Policy Evolution

- This policy SHOULD be reviewed at least once per quarter.
- Revisions MUST include rationale and compatibility notes.
- Policy version history MUST be maintained in source control.
