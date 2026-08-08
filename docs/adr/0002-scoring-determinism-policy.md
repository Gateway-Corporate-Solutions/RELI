# ADR-0002: Scoring and Settlement Determinism

- Status: Accepted
- Date: 2026-08-07
- Owners: Core protocol team
- Related roadmap phase: Phase 0
- Related policy sections: docs/design.md sections 6, 8, 10, 15

## Context

RELI rewards and slashing outcomes depend on scoring functions. Non-deterministic scoring would create inconsistent settlement outcomes across independent nodes and undermine trust.

## Decision

1. Reliability score and payout logic must be deterministic for a given input set.
2. Scoring formulas are versioned and tied to metric profiles.
3. Floating-point usage is permitted only where reproducibility tests pass across target environments.
4. Any profile that fails reproducibility is migrated to fixed-point implementation.

## Consequences

Positive:

- Settlement can be independently recomputed and audited.
- Reward fairness is enforceable.

Negative:

- May require additional engineering for fixed-point paths.
- Adds test burden for cross-environment reproducibility.

Neutral:

- Slower profile evolution due to compatibility guarantees.

## Compliance Impact

This ADR supports deterministic and reproducible settlement, transparency in incentives, and release-blocker test conditions.

## Follow-up Actions

1. Add property tests for score monotonicity and payout normalization.
2. Add reproducibility test matrix for Linux host variants.
3. Define migration policy from floating-point to fixed-point per metric profile.

## Review Sign-off

- Review date: 2026-08-07
- Decision: Accepted for Phase 0 baseline
- Approvers: Architecture Working Group (recorded in roadmap execution status)
