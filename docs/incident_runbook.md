# Pilot Incident and Rollback Runbook

## Scope

This runbook governs incident handling during the Phase 4 pilot.

## Severity Levels

1. Sev-1 Critical:
   - Safety-impacting or data-integrity-critical failure
   - Immediate rollback path required
2. Sev-2 Major:
   - Significant reliability degradation without safety impact
3. Sev-3 Minor:
   - Localized or transient issues with no broad reliability impact

## Detection Inputs

- Latency SLO alert breaches
- Spike in challenge acceptance/slashing rates
- Hash mismatch or attestation verification failures
- Event index reconstruction mismatch

## Immediate Actions (First 15 Minutes)

1. Freeze new job intake for affected profile.
2. Preserve logs, event stream snapshot, and artifact references.
3. Classify incident severity.
4. Assign incident commander and communications owner.

## Rollback Procedure

1. Revert to last known-good algorithm/profile release.
2. Disable affected worker class or profile via policy flag.
3. Re-run deterministic validation suite.
4. Resume intake only after pass criteria are met.

## Evidence Preservation

- Keep immutable event logs
- Keep challenge/dispute artifacts and signatures
- Record timeline with UTC timestamps

## Exit Criteria for Incident Closure

1. Root cause identified and documented.
2. Corrective patch validated in test and dry-run.
3. No unresolved critical incidents remain.
4. Post-incident review completed with action items.
