# Onboarding API Examples (Phase 5)

## Requester Flow (Rust Sketch)

```rust
use reli::sdk::RequesterClient;
use reli::worker::models::JobSpec;

let mut client = RequesterClient::new();
let job_id = client.create_job(JobSpec { /* fields */ }).unwrap();
let state = client.get_job_state(&job_id);
```

## Contract Lifecycle Flow (Rust Sketch)

```rust
use std::collections::HashMap;
use reli::sdk::ContractLifecycleClient;
use reli::settlement::{QualitySignal, ScoreWeights};

let mut contract = ContractLifecycleClient::new();
contract.register_worker("worker-a", 100);
contract.register_worker("worker-b", 100);

// create_job -> commit -> reveal -> verify -> finalize -> settle
```

## Governance Rollout Flow (Rust Sketch)

```rust
use reli::governance::{GovernanceChange, GovernanceEngine};

let mut gov = GovernanceEngine::new(2);
let id = gov.submit_proposal(
    "adjust quality floor",
    GovernanceChange {
        key: "quality_floor".to_string(),
        value: 0.8,
        rollback_value: 0.7,
    },
);
gov.schedule_proposal(&id, 2).unwrap();
gov.advance_epochs(2);
gov.activate_due_proposals();
```

## Capacity Benchmark Flow (Rust Sketch)

```rust
use reli::benchmarks::{BenchmarkSample, evaluate_capacity_benchmark};

let report = evaluate_capacity_benchmark("industrial-v1", &samples, 0.8);
```
