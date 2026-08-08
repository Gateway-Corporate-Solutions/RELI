# ADR-0001: Off-Chain and On-Chain Boundary

- Status: Accepted
- Date: 2026-08-07
- Owners: Core protocol team
- Related roadmap phase: Phase 0
- Related policy sections: docs/design.md sections 3, 4, 19

## Context

RELI requires heavy decoding algorithms such as Viterbi, BCJR, turbo decoding, and LDPC decoding. Running these directly on-chain is cost-prohibitive and harms throughput. However, quality and provenance must still be enforceable and auditable.

## Decision

1. All heavy decoding and reconstruction remains off-chain.
2. On-chain contracts are restricted to job lifecycle, commitments, attestation references, settlement, and governance.
3. Raw payloads are off-chain only, referenced by integrity hashes.
4. Canonical result selection is computed from attested submissions and policy-defined aggregation rules.

## Consequences

Positive:

- Preserves scalability and cost viability.
- Maintains auditable provenance through hashes and signatures.
- Keeps on-chain logic narrow and security-auditable.

Negative:

- Requires robust verifier and dispute pipelines.
- Requires strong off-chain availability and artifact durability.

Neutral:

- Increases importance of schema quality and deterministic metadata.

## Compliance Impact

This ADR satisfies strict policy requirements:

- Heavy compute off-chain invariant.
- Minimal on-chain scope invariant.
- Hash-based provenance and storage boundary requirements.

## Follow-up Actions

1. Add contract invariant tests that block heavy compute scope creep.
2. Implement verifier challenge windows and dispute metadata checks.
3. Publish storage durability and retention profile for pilot deployment.

## Review Sign-off

- Review date: 2026-08-07
- Decision: Accepted for Phase 0 baseline
- Approvers: Architecture Working Group (recorded in roadmap execution status)
