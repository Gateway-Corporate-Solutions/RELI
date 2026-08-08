#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkSample {
    pub profile_id: String,
    pub worker_count: u32,
    pub jobs_processed: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapacityBenchmarkReport {
    pub profile_id: String,
    pub samples: usize,
    pub max_throughput_jps: f64,
    pub p50_throughput_jps: f64,
    pub scaling_efficiency: f64,
    pub passes_scale_target: bool,
}

fn throughput(sample: &BenchmarkSample) -> f64 {
    if sample.duration_ms == 0 {
        return 0.0;
    }
    sample.jobs_processed as f64 / (sample.duration_ms as f64 / 1000.0)
}

pub fn evaluate_capacity_benchmark(
    profile_id: &str,
    samples: &[BenchmarkSample],
    min_scaling_efficiency: f64,
) -> Option<CapacityBenchmarkReport> {
    let mut rows = samples
        .iter()
        .filter(|sample| sample.profile_id == profile_id)
        .cloned()
        .collect::<Vec<_>>();

    if rows.is_empty() {
        return None;
    }

    rows.sort_by_key(|sample| sample.worker_count);
    let throughputs = rows.iter().map(throughput).collect::<Vec<_>>();

    let max_throughput_jps = throughputs
        .iter()
        .copied()
        .fold(0.0_f64, |acc, value| acc.max(value));

    let mut sorted = throughputs.clone();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let p50_throughput_jps = sorted[sorted.len() / 2];

    let base = &rows[0];
    let base_tp = throughput(base);
    let top = rows.last().expect("rows non-empty");
    let top_tp = throughput(top);
    let worker_scale = if base.worker_count == 0 {
        0.0
    } else {
        top.worker_count as f64 / base.worker_count as f64
    };

    let scaling_efficiency = if worker_scale <= 0.0 || base_tp <= 0.0 {
        0.0
    } else {
        (top_tp / base_tp) / worker_scale
    };

    Some(CapacityBenchmarkReport {
        profile_id: profile_id.to_string(),
        samples: rows.len(),
        max_throughput_jps,
        p50_throughput_jps,
        scaling_efficiency,
        passes_scale_target: scaling_efficiency >= min_scaling_efficiency,
    })
}

#[cfg(test)]
mod tests {
    use super::{evaluate_capacity_benchmark, BenchmarkSample};

    #[test]
    fn benchmark_passes_for_near_linear_scaling() {
        let samples = vec![
            BenchmarkSample {
                profile_id: "industrial-v1".to_string(),
                worker_count: 1,
                jobs_processed: 100,
                duration_ms: 1_000,
            },
            BenchmarkSample {
                profile_id: "industrial-v1".to_string(),
                worker_count: 2,
                jobs_processed: 185,
                duration_ms: 1_000,
            },
            BenchmarkSample {
                profile_id: "industrial-v1".to_string(),
                worker_count: 4,
                jobs_processed: 340,
                duration_ms: 1_000,
            },
        ];

        let report = evaluate_capacity_benchmark("industrial-v1", &samples, 0.8)
            .expect("report should exist");
        assert!(report.passes_scale_target);
    }

    #[test]
    fn benchmark_fails_for_poor_scaling() {
        let samples = vec![
            BenchmarkSample {
                profile_id: "satellite-v1".to_string(),
                worker_count: 1,
                jobs_processed: 100,
                duration_ms: 1_000,
            },
            BenchmarkSample {
                profile_id: "satellite-v1".to_string(),
                worker_count: 4,
                jobs_processed: 180,
                duration_ms: 1_000,
            },
        ];

        let report = evaluate_capacity_benchmark("satellite-v1", &samples, 0.8)
            .expect("report should exist");
        assert!(!report.passes_scale_target);
    }
}
