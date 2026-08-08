# Capacity Benchmark Suite (Phase 5)

## Goal

Measure horizontal scaling efficiency across profile classes to validate Phase 5 scale-out readiness.

## Primary Metrics

1. Throughput (jobs/second)
2. P50 throughput by worker-count tier
3. Scaling efficiency versus baseline worker tier
4. Pass/fail against minimum scaling target

## Recommended Profile Matrix

- industrial-v1
- satellite-v1
- healthcare-v1

## Acceptance Threshold

- Minimum scaling efficiency: 0.80 for target profile class.

## Reporting

- Persist benchmark report artifacts with profile id, sample count, throughput values, and pass/fail status.
