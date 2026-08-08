# Phase 4 Pilot Baseline and KPI Targets

## Pilot Vertical

Selected vertical: Industrial vibration monitoring for predictive maintenance.

## Baseline Data Profile

- Sensor count: 120 vibration channels
- Sample rate: 1 kHz
- Job aggregation window: 10 seconds
- Typical baseline residual error: 0.40
- Typical centralized latency: 800 ms per job
- Typical centralized cost: 1.00 unit per job

## RELI Pilot KPI Targets

1. Residual error improvement:
   - Target: at least 20% reduction vs baseline
2. Cost competitiveness:
   - Target: RELI cost per job <= baseline cost per job
3. Latency budget:
   - Target: RELI latency <= 1.25x baseline latency
4. Incident tolerance:
   - Target: zero unresolved critical incidents
5. Dispute quality:
   - Target: false-positive dispute rate <= 20%

## Acceptance Formulae

- Residual improvement % = ((baseline_residual_error - reli_residual_error) / baseline_residual_error) * 100
- Cost ratio = reli_cost_per_job / baseline_cost_per_job
- Latency ratio = reli_latency_ms / baseline_latency_ms

## Evidence Requirements

- Signed and hashed job-level outputs
- Verifier/challenger logs with dispute outcomes
- Event-indexed lifecycle records
- Incident log with severity and closure status
