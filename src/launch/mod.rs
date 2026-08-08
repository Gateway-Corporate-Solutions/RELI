#[derive(Debug, Clone, PartialEq)]
pub struct LaunchReadinessInput {
    pub legal_opinion_ready: bool,
    pub aml_kyc_policy_ready: bool,
    pub sanctions_screening_ready: bool,
    pub tokenomics_policy_finalized: bool,
    pub contract_audit_passed: bool,
    pub economic_audit_passed: bool,
    pub bug_bounty_completed: bool,
    pub governance_timelock_enabled: bool,
    pub treasury_multisig_ready: bool,
    pub testnet_canary_completed: bool,
    pub incident_drills_passed: bool,
    pub launch_communications_ready: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaunchReadinessReport {
    pub ready: bool,
    pub score_percent: f64,
    pub blockers: Vec<String>,
}

pub fn assess_launch_readiness(input: &LaunchReadinessInput) -> LaunchReadinessReport {
    let checks = [
        (
            input.legal_opinion_ready,
            "Missing legal opinion for target jurisdictions",
        ),
        (
            input.aml_kyc_policy_ready,
            "Missing AML/KYC policy package",
        ),
        (
            input.sanctions_screening_ready,
            "Missing sanctions-screening operations",
        ),
        (
            input.tokenomics_policy_finalized,
            "Tokenomics policy is not finalized",
        ),
        (
            input.contract_audit_passed,
            "Smart-contract audit not passed",
        ),
        (
            input.economic_audit_passed,
            "Economic design audit not passed",
        ),
        (
            input.bug_bounty_completed,
            "Bug bounty program not completed",
        ),
        (
            input.governance_timelock_enabled,
            "Governance timelock not enabled",
        ),
        (
            input.treasury_multisig_ready,
            "Treasury multisig not ready",
        ),
        (
            input.testnet_canary_completed,
            "Testnet/canary rollout not completed",
        ),
        (
            input.incident_drills_passed,
            "Incident drills not passed",
        ),
        (
            input.launch_communications_ready,
            "Launch communications and disclosures not ready",
        ),
    ];

    let passed = checks.iter().filter(|(ok, _)| *ok).count();
    let total = checks.len();
    let blockers = checks
        .iter()
        .filter_map(|(ok, message)| if *ok { None } else { Some((*message).to_string()) })
        .collect::<Vec<_>>();

    LaunchReadinessReport {
        ready: blockers.is_empty(),
        score_percent: (passed as f64 / total as f64) * 100.0,
        blockers,
    }
}

#[cfg(test)]
mod tests {
    use super::{assess_launch_readiness, LaunchReadinessInput};

    fn all_true() -> LaunchReadinessInput {
        LaunchReadinessInput {
            legal_opinion_ready: true,
            aml_kyc_policy_ready: true,
            sanctions_screening_ready: true,
            tokenomics_policy_finalized: true,
            contract_audit_passed: true,
            economic_audit_passed: true,
            bug_bounty_completed: true,
            governance_timelock_enabled: true,
            treasury_multisig_ready: true,
            testnet_canary_completed: true,
            incident_drills_passed: true,
            launch_communications_ready: true,
        }
    }

    #[test]
    fn launch_is_ready_when_all_gates_pass() {
        let report = assess_launch_readiness(&all_true());
        assert!(report.ready);
        assert_eq!(report.blockers.len(), 0);
        assert!((report.score_percent - 100.0).abs() < 1e-12);
    }

    #[test]
    fn launch_is_blocked_when_critical_items_fail() {
        let mut input = all_true();
        input.contract_audit_passed = false;
        input.legal_opinion_ready = false;

        let report = assess_launch_readiness(&input);
        assert!(!report.ready);
        assert!(report.blockers.iter().any(|msg| msg.contains("legal opinion")));
        assert!(report
            .blockers
            .iter()
            .any(|msg| msg.contains("Smart-contract audit")));
        assert!(report.score_percent < 100.0);
    }
}
