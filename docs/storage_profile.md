# Artifact Storage Retention and Durability Profile (Phase 2)

## Scope

This profile defines retention and durability expectations for noisy inputs, worker outputs, and evidence artifacts referenced by RELI jobs.

## Artifact Classes

1. Raw noisy input artifacts
2. Candidate output artifacts
3. Soft evidence bundles
4. Verification and challenge proof artifacts

## Storage Policy

- Raw payloads are off-chain and content-addressed.
- On-chain references store only hash/cid pointers and lifecycle metadata.
- Every artifact reference must include an integrity digest.

## Retention Windows

1. Active job window:
   - Keep all artifacts online and queryable through settle finality plus dispute window.
2. Post-settlement audit window:
   - Retain full artifacts for at least 180 days.
3. Long-term archive:
   - Keep hash attestations indefinitely.
   - Keep compressed/coded artifact snapshots for at least 730 days.

## Durability Targets

1. Hot storage durability target: 11 nines equivalent (or provider SLA equivalent).
2. Archive durability target: geo-redundant, multi-region replication.
3. Availability target for audit retrieval: 99.9% monthly.

## Integrity and Recovery

- Run periodic hash re-verification of stored objects.
- On mismatch, quarantine and rebuild from redundant replica set.
- Recovery runbook must restore availability within 4 hours for active jobs.

## Access and Privacy

- Sensitive artifacts require encrypted-at-rest storage.
- Access control is role-scoped and auditable.
- Every retrieval and mutation action must emit a signed audit event.

## Operational Controls

1. Weekly integrity sweep for active artifacts.
2. Monthly archive verification sample.
3. Quarterly disaster recovery drill for artifact restoration.

## Phase 2 Acceptance Mapping

- Supports contract lifecycle auditability by ensuring evidence artifacts remain available through dispute and audit windows.
- Supports deterministic replay and investigation via durable event + artifact linkage.
