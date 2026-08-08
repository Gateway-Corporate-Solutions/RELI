#[derive(Debug, Clone, Copy)]
pub struct PilotMetrics {
    pub baseline_residual_error: f64,
    pub reli_residual_error: f64,
    pub baseline_cost_per_job: f64,
    pub reli_cost_per_job: f64,
    pub baseline_latency_ms: u64,
    pub reli_latency_ms: u64,
    pub critical_incident_count: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct PilotThresholds {
    pub min_residual_improvement_pct: f64,
    pub max_cost_ratio_vs_baseline: f64,
    pub max_latency_ratio_vs_baseline: f64,
    pub max_critical_incidents: u32,
}

#[derive(Debug, Clone)]
pub struct PilotAssessment {
    pub residual_improvement_pct: f64,
    pub cost_ratio_vs_baseline: f64,
    pub latency_ratio_vs_baseline: f64,
    pub reliability_pass: bool,
    pub cost_pass: bool,
    pub latency_pass: bool,
    pub incidents_pass: bool,
    pub overall_pass: bool,
}

pub fn assess_pilot(metrics: PilotMetrics, thresholds: PilotThresholds) -> PilotAssessment {
    let residual_improvement_pct = if metrics.baseline_residual_error <= 0.0 {
        0.0
    } else {
        ((metrics.baseline_residual_error - metrics.reli_residual_error)
            / metrics.baseline_residual_error)
            * 100.0
    };

    let cost_ratio_vs_baseline = if metrics.baseline_cost_per_job <= 0.0 {
        f64::INFINITY
    } else {
        metrics.reli_cost_per_job / metrics.baseline_cost_per_job
    };

    let latency_ratio_vs_baseline = if metrics.baseline_latency_ms == 0 {
        f64::INFINITY
    } else {
        metrics.reli_latency_ms as f64 / metrics.baseline_latency_ms as f64
    };

    let reliability_pass = residual_improvement_pct >= thresholds.min_residual_improvement_pct;
    let cost_pass = cost_ratio_vs_baseline <= thresholds.max_cost_ratio_vs_baseline;
    let latency_pass = latency_ratio_vs_baseline <= thresholds.max_latency_ratio_vs_baseline;
    let incidents_pass = metrics.critical_incident_count <= thresholds.max_critical_incidents;

    PilotAssessment {
        residual_improvement_pct,
        cost_ratio_vs_baseline,
        latency_ratio_vs_baseline,
        reliability_pass,
        cost_pass,
        latency_pass,
        incidents_pass,
        overall_pass: reliability_pass && cost_pass && latency_pass && incidents_pass,
    }
}

#[cfg(test)]
mod tests {
    use super::{assess_pilot, PilotMetrics, PilotThresholds};

    fn thresholds() -> PilotThresholds {
        PilotThresholds {
            min_residual_improvement_pct: 20.0,
            max_cost_ratio_vs_baseline: 1.0,
            max_latency_ratio_vs_baseline: 1.25,
            max_critical_incidents: 0,
        }
    }

    #[test]
    fn pilot_assessment_passes_when_all_phase4_criteria_are_met() {
        let metrics = PilotMetrics {
            baseline_residual_error: 0.40,
            reli_residual_error: 0.28,
            baseline_cost_per_job: 1.00,
            reli_cost_per_job: 0.95,
            baseline_latency_ms: 800,
            reli_latency_ms: 900,
            critical_incident_count: 0,
        };

        let assessment = assess_pilot(metrics, thresholds());
        assert!(assessment.overall_pass);
        assert!(assessment.reliability_pass);
        assert!(assessment.cost_pass);
        assert!(assessment.latency_pass);
        assert!(assessment.incidents_pass);
    }

    #[test]
    fn pilot_assessment_fails_when_reliability_gain_is_too_small() {
        let metrics = PilotMetrics {
            baseline_residual_error: 0.40,
            reli_residual_error: 0.35,
            baseline_cost_per_job: 1.00,
            reli_cost_per_job: 0.95,
            baseline_latency_ms: 800,
            reli_latency_ms: 900,
            critical_incident_count: 0,
        };

        let assessment = assess_pilot(metrics, thresholds());
        assert!(!assessment.reliability_pass);
        assert!(!assessment.overall_pass);
    }

    #[test]
    fn pilot_assessment_fails_with_critical_incident() {
        let metrics = PilotMetrics {
            baseline_residual_error: 0.40,
            reli_residual_error: 0.28,
            baseline_cost_per_job: 1.00,
            reli_cost_per_job: 0.95,
            baseline_latency_ms: 800,
            reli_latency_ms: 900,
            critical_incident_count: 1,
        };

        let assessment = assess_pilot(metrics, thresholds());
        assert!(!assessment.incidents_pass);
        assert!(!assessment.overall_pass);
    }
}
