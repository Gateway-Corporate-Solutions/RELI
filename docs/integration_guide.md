# Partner Integration Guide (Phase 5)

## 1. Overview

This guide describes how external partners integrate with RELI for reliability-as-a-service workflows.

## 2. Integration Steps

1. Select metric profile for your vertical.
2. Register requester and worker identities.
3. Submit jobs with noisy input references and acceptance policy.
4. Monitor job lifecycle events.
5. Retrieve finalized outputs and attestation records.

## 3. Recommended Onboarding Sequence

1. Sandbox dry run with fixture inputs.
2. Pilot-mode run with dispute handling enabled.
3. Production-mode run after KPI and incident criteria are met.

## 4. Required Operational Capabilities

- Ability to store and retrieve off-chain artifacts by hash/cid.
- Ability to validate worker signatures and event timelines.
- Ability to process challenge and settlement outcomes.

## 5. Verification Checklist

- Job output hash matches attested reference.
- Submission signatures validate.
- Event timeline reconstructs to settled state.
- Payout and slashing events are reproducible.

## 6. Support Channels

- Protocol issues: architecture/governance review process.
- Integration issues: SDK and schema compatibility review.
